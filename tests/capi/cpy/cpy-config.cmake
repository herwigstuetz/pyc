add_library(cpy STATIC IMPORTED GLOBAL)

set_target_properties(cpy PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_LIST_DIR}/../../../target/debug/libpy.a"
#    IMPORTED_LINK_INTERFACE_LIBRARIES "-lpython3.8"
    INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_CURRENT_LIST_DIR}/../../../target/debug/")
