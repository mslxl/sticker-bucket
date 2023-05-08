cmake_minimum_required(VERSION 3.25)
project(mmm)

set(CMAKE_CXX_STANDARD 23)

set(source_list
        src/bridge.h src/bridge.cpp
        src/ocr/text_angle_cls.cpp src/ocr/text_angle_cls.h src/ocr/text_det.cpp src/ocr/text_det.h src/ocr/text_rec.cpp src/ocr/text_rec.h)
set(CMAKE_CXX_FLAGS -m64)
find_package(OpenCV CONFIG REQUIRED)


add_library(${PROJECT_NAME} STATIC
        ${source_list})
include_directories(3rd_party/ort/include)
link_directories(3rd_party/ort/lib)
target_link_libraries(${PROJECT_NAME} PUBLIC onnxruntime)
target_link_libraries(${PROJECT_NAME} PUBLIC ${OpenCV_LIBS})

add_executable(hello ${source_list} tests/hello.cpp)
target_link_libraries(hello PRIVATE ${PROJECT_NAME})