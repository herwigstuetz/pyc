use std::{collections::HashMap, ffi::{c_void, CString}, os::raw::c_char};
use std::{ffi::CStr, fs};

use pyo3::types::{IntoPyDict, PyDict};
use pyo3::{exceptions::PyIndexError, prelude::*};
use pyo3::prelude::PyModule;
use pyo3::conversion::ToPyObject;

type Ports = HashMap<String, Value>;

#[derive(Debug)]
enum Value {
    Float(f64),
    Integer(isize),
    Bool(bool),
}

impl ToPyObject for Value {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            Value::Float(d) => {d.into_py(py)}
            Value::Integer(i) => {i.into_py(py)}
            Value::Bool(b) => {b.into_py(py)}
        }
    }
}

// The python representation of `Value` is a `PyDic` with the keys
// `type` and `value` and the value of `value` the corresponding
// python type to `type`.
impl<'source> FromPyObject<'source> for Value {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let dict = ob.downcast::<PyDict>()?;
        if let Some(ty) = dict.get_item("type") {
            let val = dict
                .get_item("value")
                .ok_or_else(|| PyIndexError::new_err("No key 'value'"))?;

            if ty.extract::<String>()? == "float" {
                return Ok(Value::Float(val.extract::<f64>()?));
            } else if ty.extract::<String>()? == "int" {
                return Ok(Value::Integer(val.extract::<isize>()?));
            } else if ty.extract::<String>()? == "bool" {
                return Ok(Value::Bool(val.extract::<bool>()?));
            }
        };

        Err(PyIndexError::new_err("No key 'type'"))
    }
}

struct CPy {
    _module: Py<PyModule>,
    _locals: Py<PyDict>,

    // instance of class in module
    instance: PyObject,
}

struct CPyModule {
    content: String,
    module_name: String,
    file_name: String,
}

#[repr(C)]
#[derive(Debug)]
enum CPyPortType {
    Float,
    Int,
    Bool,
}

#[repr(C)]
union CPyPortValue {
    d: f64,
    i: isize,
    b: bool,
}

#[repr(C)]
pub struct CPyPort {
    name: *const c_char,
    type_: CPyPortType,
    value: CPyPortValue,
}

impl std::fmt::Debug for CPyPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { f.write_str(&format!("CPyPort {{ name: {}, type: {}, value: {}}}", &CStr::from_ptr(self.name).to_str().unwrap(), 0, self.value.d)) }
    }
}

impl CPy {
    fn new(py: Python, module: CPyModule, class_name: &str) -> Result<Self, String> {
        let py_module =
            PyModule::from_code(py, &module.content, &module.file_name, &module.module_name)
            .map_err(|_| format!("Could not load module {}", &module.module_name))?;

        let py_locals = [
            ("os", py.import("os").map_err(|e| e.to_string())?),
            (&module.module_name, py_module),
        ]
            .into_py_dict(py);

        let py_instance = py
            .eval(
                &format!("{}.{}()", &module.module_name, &class_name),
                None,
                Some(&py_locals),
            )
            .map_err(|_| "Could not eval code".to_string())?;

        Ok(CPy {
            // unused for now because existing code goes through instance
            _module: py_module.into(),
            _locals: py_locals.into(),

            instance: py_instance.into(),
        })
    }

    fn configure(&self) -> Result<(), String> {
        let res = Python::with_gil(|py| -> PyResult<PyObject> {
            self.instance.call_method0(py, "configure")
        });

        match res {
            Ok(_obj) => Ok(()), // ignore result of configure
            Err(err) => Err(err.to_string()),
        }
    }

    fn run(&self) -> Result<(), String> {
        let res = Python::with_gil(|py| -> PyResult<PyObject> {
            self.instance.call_method0(py, "run")
        });

        match res {
            Ok(_obj) => Ok(()), // ignore result of run
            Err(err) => Err(err.to_string()),
        }
    }

    fn get(&self) -> Result<Ports, String> {
        let ports: Result<Ports, String> = Python::with_gil(|py| -> Result<Ports, String> {
            // outports as PyObject
            let outports = self
                .instance
                .getattr(py, "outports")
                .map_err(|e| e.to_string())?;

            // outports as PyDict
            let outports = outports
                .as_ref(py)
                .downcast::<PyDict>()
                .map_err(|e| e.to_string())?;

            // outports as Ports
            outports.extract().map_err(|e| e.to_string())
        });

        ports
    }

    fn set(&self, ports: Ports) -> Result<(), String> {
        Python::with_gil(|py| -> Result<(), String> {
            let outports = self
                .instance
                .getattr(py, "outports")
                .map_err(|e| e.to_string())?;
            let outports = outports
                .as_ref(py)
                .downcast::<PyDict>()
                .map_err(|e| e.to_string())?;

            for (name, value) in ports {
                #[allow(clippy::single_match)]
                match outports.get_item(name) {
                    Some(py_port) => {
                        let py_dict = py_port.downcast::<PyDict>().map_err(|e| e.to_string())?;

                        if let Err(e) = py_dict.set_item("value", value) {
                            return Err(e.to_string());
                        }
                    },
                    None => {
                        // ignore port from Ports that is not present in python
                    },
                }
            }

            Ok(())
        })
    }
}

#[no_mangle]
pub extern "C" fn cpy_new(
    file_name: *const c_char,
    module_name: *const c_char,
    class_name: *const c_char,
) -> *mut c_void {
    if file_name.is_null() {
        return std::ptr::null_mut();
    }

    if module_name.is_null() {
        return std::ptr::null_mut();
    }

    if class_name.is_null() {
        return std::ptr::null_mut();
    }

    // safe because is_null() check
    let file_name = match unsafe { CStr::from_ptr(file_name).to_str() } {
        Ok(file_name) => file_name,
        Err(_) => return std::ptr::null_mut(),
    };

    // safe because is_null() check
    let module_name = match unsafe { CStr::from_ptr(module_name).to_str() } {
        Ok(module_name) => module_name,
        Err(_) => return std::ptr::null_mut(),
    };

    // safe because is_null() check
    let class_name = match unsafe { CStr::from_ptr(class_name).to_str() } {
        Ok(class_name) => class_name,
        Err(_) => return std::ptr::null_mut(),
    };

    let content = fs::read_to_string(file_name).expect("Something went wrong reading the file");

    let res: Result<CPy, String> = Python::with_gil(|py| {
        CPy::new(
            py,
            CPyModule {
                content,
                module_name: module_name.to_string(),
                file_name: file_name.to_string(),
            },
            &class_name,
        )
    });

    match res {
        Ok(cpy) => Box::into_raw(Box::new(cpy)) as *mut _,
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn cpy_configure(c_cpy: *mut c_void) -> i32 {
    if c_cpy.is_null() {
        return 1;
    }

    // safe because is_null() check
    let cpy: Box<CPy> = unsafe { Box::from_raw(c_cpy as *mut _) };

    match cpy.configure() {
        Ok(()) => {
            Box::into_raw(cpy);
            0
        },
        Err(e) => {
            println!("{}", e);
            1
        }
    }
}

#[no_mangle]
pub extern "C" fn cpy_run(c_cpy: *mut c_void) -> i32 {
    if c_cpy.is_null() {
        return 1;
    }

    // safe because is_null() check
    let cpy: Box<CPy> = unsafe { Box::from_raw(c_cpy as *mut _) };

    match cpy.run() {
        Ok(()) => {
            Box::into_raw(cpy);
            0
        },
        Err(e) => {
            println!("{}", e);
            1
        }
    }
}

fn as_c_string(s: String) -> *mut c_char {
    CString::new(s)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut() as *mut i8)
}

fn as_c_port_type(value: &Value) -> CPyPortType {
    match value {
        Value::Float(_) => CPyPortType::Float,
        Value::Integer(_) => CPyPortType::Int,
        Value::Bool(_) => CPyPortType::Bool,
    }
}

fn as_c_port_value(value: &Value) -> CPyPortValue {
    match value {
        Value::Float(d) => CPyPortValue { d: *d },
        Value::Integer(i) => CPyPortValue { i: *i },
        Value::Bool(b) => CPyPortValue { b: *b },
    }
}

fn as_c_ports(ports: &Ports) -> std::vec::Vec<CPyPort> {
    let mut v = vec![];

    for (name, port) in ports {
        let c_port = CPyPort {
            name: as_c_string(name.clone()),
            type_: as_c_port_type(port),
            value: as_c_port_value(port),
        };

        v.push(c_port);
    }

    v
}

unsafe fn from_c_port(c_port: CPyPort) -> Value {
    match c_port.type_ {
        CPyPortType::Float => Value::Float(c_port.value.d),
        CPyPortType::Int => Value::Integer(c_port.value.i),
        CPyPortType::Bool => Value::Bool(c_port.value.b),
    }
}

#[no_mangle]
pub extern "C" fn cpy_get(c_cpy: *mut c_void, c_ports: *mut *mut CPyPort, c_num: *mut usize) -> i32 {
    if c_cpy.is_null() {
        return 1;
    }

    if c_ports.is_null() {
        return 1;
    }

    if c_num.is_null() {
        return 1;
    }

    // safe because is_null() check
    let cpy: Box<CPy> = unsafe { Box::from_raw(c_cpy as *mut _) };

    let ports = cpy.get();

    match ports {
        Ok(ports) => {
            let ports_slice = as_c_ports(&ports).into_boxed_slice();

            unsafe {
                std::ptr::write(c_num, ports.len());
                std::ptr::write(c_ports, Box::into_raw(ports_slice) as *mut _);
            }
            Box::into_raw(cpy);

            0
        },
        Err(e) => {
            println!("{}", e);
            1
        }
    }
}

#[no_mangle]
//pub extern "C" fn cpy_free_ports(c_cpy: *mut c_void, c_ports: *mut CPyPort, c_num: usize) -> i32 {
pub extern "C" fn cpy_free_ports(c_ports: *mut CPyPort) {
    if c_ports.is_null() {
        return;
    }

    unsafe { Box::from_raw(c_ports); }
}

#[no_mangle]
pub extern "C" fn cpy_set(c_cpy: *mut c_void, c_ports: *mut CPyPort, c_num: usize) -> i32 {
    if c_cpy.is_null() {
        return 1;
    }

    if c_ports.is_null() {
        return 1;
    }

    // safe because is_null() check
    let cpy: Box<CPy> = unsafe { Box::from_raw(c_cpy as *mut _) };

    let ports = unsafe { Vec::from_raw_parts(c_ports, c_num, c_num) };
    let ports = {
        let mut dict = HashMap::<String, Value>::new();
        for port in ports {
            let name = match unsafe { CStr::from_ptr(port.name).to_str() } {
                Ok(name) => name,
                Err(_) => {return 1;} //Err("Invalid name".to_string()),
            };
            let value = unsafe { from_c_port(port) };

            dict.insert(name.to_string(), value);
        };

        dict
    };

    match cpy.set(ports) {
        Ok(()) => {
            Box::into_raw(cpy);
            0
        },
        Err(e) => {
            println!("{}", e);
            1
        }
    }
}
