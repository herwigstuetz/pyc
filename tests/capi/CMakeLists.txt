project(cpy-c)
cmake_minimum_required(VERSION 3.12)

find_package(cpy REQUIRED)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

add_executable(
  ${PROJECT_NAME}
  src/main.c
  )

target_link_libraries (
  ${PROJECT_NAME}
  PRIVATE
  cpy
  )

install(TARGETS ${PROJECT_NAME})
