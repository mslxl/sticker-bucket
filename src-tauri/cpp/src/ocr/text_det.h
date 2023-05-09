//
// Created by mslxl on 5/8/2023.
//

#ifndef MMM_TEXT_DET_H
#define MMM_TEXT_DET_H

#include <iostream>
#include <opencv2/imgproc.hpp>
#include <onnxruntime_cxx_api.h>


class TextDetector{
public:
    explicit TextDetector(const std::string& model_path);
    ~TextDetector();
    std::vector<std::vector<cv::Point2f>> detect(cv::Mat& src);
    void draw_pred(cv::Mat & src, std::vector<std::vector<cv::Point2f>> results);
    cv::Mat get_rotate_crop_image(const cv::Mat& frame, std::vector<cv::Point2f> vertices);

private:
    float binary_threshold;
    float polygon_threshold;
    float unclip_ratio;
    float max_candidates;
    const int long_side_thresh = 3; // minBox 长边门限
    const int short_size = 736;

    const float mean_values[3] = {0.485, 0.456, 0.406};
    const float norm_values[3] = {0.229, 0.224, 0.225};

    float contour_score(const cv::Mat& binary, const std::vector<cv::Point> & contour);
    void unclip(const std::vector<cv::Point2f> &in_poly, std::vector<cv::Point2f> &out_poly);
    std::vector<std::vector<cv::Point2f >> order_points_clockwise(std::vector<std::vector<cv::Point2f>> result);

    std::vector<float> input_image;
    cv::Mat preprocess(const cv::Mat& src);
    void normalize(cv::Mat img);

    Ort::Session *net;
    Ort::Env env = Ort::Env(ORT_LOGGING_LEVEL_WARNING, "DBNet");
    Ort::SessionOptions session_options = Ort::SessionOptions();

    std::vector<char*> input_names;
    std::vector<char*> output_names;
    Ort::AllocatorWithDefaultOptions allocator;
};


#endif //MMM_TEXT_DET_H
