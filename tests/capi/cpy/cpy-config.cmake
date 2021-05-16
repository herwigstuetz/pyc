add_library(cpy STATIC IMPORTED GLOBAL)

set_target_properties(cpy PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_LIST_DIR}/../../../target/debug/libpy.a"
#    IMPORTED_LINK_INTERFACE_LIBRARIES "-lpython3.8"


add_library(cpy-dynamic SHARED IMPORTED GLOBAL)

set_target_properties(cpy-dynamic PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_LIST_DIR}/../../../target/debug/libpy.so"
    # python is loaded explicitly
    # IMPORTED_LINK_INTERFACE_LIBRARIES "-lpython3.8"
    INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_CURRENT_LIST_DIR}/../../../target/debug/")
