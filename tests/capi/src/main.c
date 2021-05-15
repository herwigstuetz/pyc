#include "stdio.h"
#include "cpy.h"

//char* PyByteArray_AsString() {
//    return NULL;
//}

int main() {
    CPyPort *ports;
    size_t num;

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
