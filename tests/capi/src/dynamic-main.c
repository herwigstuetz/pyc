#include <stdio.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#include <dlfcn.h>

typedef enum {
  Float,
  Int,
  Bool,
} CPyPortType;

typedef union {
  double d;
  intptr_t i;
  bool b;
} CPyPortValue;

typedef struct {
  const char *name;
  CPyPortType type_;
  CPyPortValue value;
} CPyPort;


typedef void *(*fcpy_new)(const char *file_name, const char *module_name, const char *class_name);
typedef int32_t (*fcpy_configure)(void *c_cpy);
typedef int32_t (*fcpy_run)(void *c_cpy);
typedef int32_t (*fcpy_get)(void *c_cpy, CPyPort **c_ports, uintptr_t *c_num);
typedef int32_t (*fcpy_set)(void *c_cpy, CPyPort *c_ports, uintptr_t c_num);
typedef void (*fcpy_free_ports)(CPyPort *c_ports);

fcpy_new cpy_new;
fcpy_configure cpy_configure;
fcpy_run cpy_run;
fcpy_get cpy_get;
fcpy_set cpy_set;
fcpy_free_ports cpy_free_ports;

int main() {
    CPyPort *ports;
    size_t num;

    // load python with RTLD_GLOBAL so that libpy finds the python symbols
    void * pyso = dlopen("./libpython3.6m.so.1.0", RTLD_GLOBAL | RTLD_NOW);

    void * so = dlopen("./target/debug/libpy.so", RTLD_LOCAL | RTLD_NOW);

    cpy_new = dlsym(so, "cpy_new");
    cpy_configure = dlsym(so, "cpy_configure");
    cpy_run = dlsym(so, "cpy_run");
    cpy_get = dlsym(so, "cpy_get");
    cpy_set = dlsym(so, "cpy_set");
    cpy_free_ports = dlsym(so, "cpy_free_ports");


    void *py = cpy_new("module.py", "module", "Abc");
    if (!py) {  printf("error new\n"); }

    if (cpy_configure(py)) {  printf("error configure\n"); }
    if (cpy_get(py, &ports, &num)) {  printf("error get\n"); }

    for (size_t i = 0; i < num; i++) {
        printf("%s: %d, %f\n", ports[i].name, ports[i].type_, ports[i].value.d);
    }

    if (cpy_run(py)) {  printf("error run\n"); }
    if (cpy_get(py, &ports, &num)) {  printf("error get\n"); }
    for (size_t i = 0; i < num; i++) {
        printf("%s: %d, %f\n", ports[i].name, ports[i].type_, ports[i].value.d);
    }

    for (size_t i = 0; i < num; i++) {
        ports[i].value.d -= 10.0;
    }
    if (cpy_set(py, ports, num)) {
        printf("error set\n");
    }

    if (cpy_run(py)) {
        printf("error run\n");
    }
    if (cpy_get(py, &ports, &num)) {
        printf("error get\n");
    }
    for (size_t i = 0; i < num; i++) {
        printf("%s: %d, %f\n", ports[i].name, ports[i].type_, ports[i].value.d);
    }

    return 0;
}
