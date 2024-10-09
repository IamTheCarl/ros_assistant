cmake_minimum_required(VERSION 3.8)
project(create_bridge)

find_package(ament_cmake REQUIRED)

install(PROGRAMS
  # $ENV{EXECUTABLE}
  ${CMAKE_SOURCE_DIR}/create_bridge
  DESTINATION lib/${PROJECT_NAME}
)

ament_package()
