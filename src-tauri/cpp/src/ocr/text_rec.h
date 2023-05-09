//
// Created by mslxl on 5/8/2023.
//

#ifndef MMM_TEXT_REC_H
#define MMM_TEXT_REC_H
#include<string>
#include<opencv2/imgproc.hpp>
#include<onnxruntime_cxx_api.h>
class TextRecognizer
{
public:
    TextRecognizer(const std::string& model_name, const std::string& dict);
    std::string predict_text(cv::Mat cv_image);

private:
    cv::Mat preprocess(cv::Mat srcimg);
    void normalize(cv::Mat img);
    const int inpWidth = 320;
    const int inpHeight = 48;

    std::vector<float> input_image_;
    std::vector<std::string> alphabet;
    int names_len;
    std::vector<int> preb_label;

    Ort::Env env = Ort::Env(ORT_LOGGING_LEVEL_ERROR, "CRNN");
    Ort::Session *ort_session = nullptr;
    Ort::SessionOptions sessionOptions = Ort::SessionOptions();
    std::vector<char*> input_names;
    std::vector<char*> output_names;
    std::vector<std::vector<int64_t>> input_node_dims; // >=1 outputs
    std::vector<std::vector<int64_t>> output_node_dims; // >=1 outputs
    Ort::AllocatorWithDefaultOptions allocator;
};
#endif //MMM_TEXT_REC_H
