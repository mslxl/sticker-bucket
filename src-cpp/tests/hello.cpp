#include <iostream>
#include <opencv2/opencv.hpp>
#include <onnxruntime_cxx_api.h>
#include "../src/ocr/text_angle_cls.h"
#include "../src/ocr/text_det.h"
#include "../src/ocr/text_rec.h"

void test_image(){
    auto icon = cv::imread("../../src-tauri/icons/icon.png");
    cv::imshow("Icon", icon);
    cv::waitKey();
}

void test_onnx(){
    Ort::Env env(ORT_LOGGING_LEVEL_WARNING, "test");
    Ort::SessionOptions session_option;
    session_option.SetInterOpNumThreads(1);

    session_option.SetGraphOptimizationLevel(GraphOptimizationLevel::ORT_ENABLE_EXTENDED);

#ifdef _WIN32
    const wchar_t* model_path = L"../model/ch_PP-OCRv3_det_infer.onnx";
#else
    const char* model_path = "../model/ch_PP-OCRv3_det_infer.onnx";
#endif

    Ort::Session session(env, model_path, session_option);
    Ort::AllocatorWithDefaultOptions allocator;

    auto input_nodes = session.GetInputCount();
    for(auto idx : std::views::iota(0,int(input_nodes))){
        auto name = session.GetInputNameAllocated(idx, allocator);
        std::cout << *name << std::endl;
    }
}

void test_ocr(){
    TextDetector detect_model("model/ch_PP-OCRv3_det_infer.onnx");
    TextClassifier angle_model("model/ch_ppocr_mobile_v2.0_cls.onnx");
    TextRecognizer rec_model("model/ch_PP-OCRv3_rec_infer.onnx", "model/rec_word_dict.txt");


    std::string image_path = "../test_image.jpg";
    cv::Mat src = cv::imread(image_path);

    std::vector<std::vector<cv::Point2f>> result = detect_model.detect(src);

    for(std::size_t i = 0; i < result.size(); i++){
        cv::Mat textimg = detect_model.get_rotate_crop_image(src, result[i]);
        cv::imshow("text", textimg);
        cv::waitKey();

        if(angle_model.predict(textimg)){
            cv::rotate(textimg, textimg, 1);
        }
        std::string text = rec_model.predict_text(textimg);
        std::cout <<text << std::endl;
    }
}

int main() {
#define named_function_ptr(name) std::make_pair(#name, name)
    auto ptr= std::vector({
                                  named_function_ptr(test_image),
                                  named_function_ptr(test_onnx),
                                  named_function_ptr(test_ocr)
    });
    std::cout << "Input testcase:" << std::endl;
    for(std::size_t idx = 0; auto& name : ptr){
        std::cout << idx << ". " << name.first << std::endl;
        idx++;
    }
    int option = 0;
    std::cin >> option;
    std::cout << "--- " << ptr[option].first << " ---" << std::endl;
    ptr[option].second();
    return 0;
}
