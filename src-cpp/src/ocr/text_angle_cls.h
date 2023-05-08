//
// Created by mslxl on 5/7/2023.
//

#ifndef MMM_TEXT_ANGLE_CLS_H
#define MMM_TEXT_ANGLE_CLS_H

#include <iostream>
#include <fstream>
#include <numeric>
#include <opencv2/imgproc.hpp>
#include <opencv2/highgui.hpp>
#include <onnxruntime_cxx_api.h>


class TextClassifier{
public:
    explicit TextClassifier(const std::string& model_path);
    ~TextClassifier();
    int predict(cv::Mat image);
private:
    const int inp_width = 192;
    const int inp_height = 48;
    const int label_list[2] = {0, 180};

    int num_out;
    std::vector<float> input_image;
    Ort::Env env = Ort::Env(ORT_LOGGING_LEVEL_WARNING, "Angle classify");
    Ort::Session *ort_session = nullptr;
    Ort::SessionOptions session_option = Ort::SessionOptions();
    Ort::AllocatorWithDefaultOptions allocator;

    std::vector<char*> input_names;
    std::vector<char*> output_names;
    std::vector<std::vector<std::int64_t>> input_node_dims;
    std::vector<std::vector<std::int64_t>> output_node_dims;

    cv::Mat preprocess(const cv::Mat& src);
    void normalize(cv::Mat img);


};

#endif //MMM_TEXT_ANGLE_CLS_H
