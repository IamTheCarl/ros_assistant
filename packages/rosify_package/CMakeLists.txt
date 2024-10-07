cmake_minimum_required(VERSION 3.8)
project($ENV{NODE_NAME})

find_package(ament_cmake REQUIRED)

install(PROGRAMS
  ${CMAKE_SOURCE_DIR}/$ENV{NODE_NAME}
  DESTINATION lib/${PROJECT_NAME}
)

ament_package()
