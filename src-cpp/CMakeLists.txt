cmake_minimum_required(VERSION 3.25)
project(mmm)

set(CMAKE_CXX_STANDARD 17)

set(source_list
        header/bridge.h src/bridge.cpp)

find_package(OpenCV CONFIG REQUIRED)


add_library(${PROJECT_NAME} SHARED
        ${source_list})
target_link_libraries(${PROJECT_NAME} PUBLIC ${OpenCV_LIBS})

add_executable(hello ${source_list} tests/hello.cpp)
target_link_libraries(hello PUBLIC ${OpenCV_LIBS})