cmake_minimum_required(VERSION 3.25)
project(mmm)

set(CMAKE_CXX_STANDARD 23)

set(source_list
        src/bridge.h src/bridge.cpp
        src/ocr/text_angle_cls.cpp src/ocr/text_angle_cls.h
        src/ocr/text_det.cpp src/ocr/text_det.h
        src/ocr/text_rec.cpp src/ocr/text_rec.h)
set(CMAKE_CXX_FLAGS -m64)

add_library(mmm STATIC
        ${source_list})

find_package(OpenCV CONFIG REQUIRED)
target_link_libraries(mmm PRIVATE ${OpenCV_LIBS})
include_directories(${OpenCV_INCLUDE_DIRS})


# ORT
include_directories(3rd_party/ort/include)
link_directories(3rd_party/ort/lib)
# windows has onnxruntime old version
# copy ort lib to avoid version conflict
file(GLOB ORT_FILES 3rd_party/ort/lib/onnxruntime.*)
file(COPY ${ORT_FILES} DESTINATION . FOLLOW_SYMLINK_CHAIN)

target_link_libraries(${PROJECT_NAME} PRIVATE onnxruntime)

add_executable(hello ${source_list} tests/hello.cpp)
target_link_libraries(hello PRIVATE ${OpenCV_LIBS})
target_link_libraries(hello PRIVATE onnxruntime)

set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR})
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR})
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR})


set(CMAKE_INSTALL_PREFIX "./lib" )

