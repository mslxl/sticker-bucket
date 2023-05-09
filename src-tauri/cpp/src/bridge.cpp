//
// Created by mslxl on 5/7/2023.
//

#include "bridge.h"
#include "ocr/text_rec.h"
#include "ocr/text_det.h"
#include "ocr/text_angle_cls.h"


const char* ocr_image(const char* image_path){
    std::cout << "recognized " << image_path << std::endl;
    TextDetector detect_model("model/ch_PP-OCRv3_det_infer.onnx");
    TextClassifier angle_model("model/ch_ppocr_mobile_v2.0_cls.onnx");
    TextRecognizer rec_model("model/ch_PP-OCRv3_rec_infer.onnx", "model/rec_word_dict.txt");

    cv::VideoCapture capture(image_path, cv::CAP_FFMPEG);

    cv::Mat src;

    if(!capture.isOpened() || !capture.read(src)){
        std::cerr << "read image " << src << " in video capture fail" << std::endl;
        src = cv::imread(image_path, cv::IMREAD_COLOR);
        if(src.data == nullptr){
            std::cerr << "read image " << src << "in normal fail" << std::endl;
            return "";
        }
    }else{
        capture.release();
    }

    std::cout << "detecting..." << std::endl;
    std::vector<std::vector<cv::Point2f>> result = detect_model.detect(src);
    std::reverse(result.begin(), result.end());
    std::cout << "found " << result.size() << " text box" << std::endl;
    std::ostringstream os;
    for(std::size_t i = 0; i < result.size(); i++){
        cv::Mat textimg = detect_model.get_rotate_crop_image(src, result[i]);

        auto angle_cls = angle_model.predict(textimg);
        if(angle_cls){
            std::cout << "predict angle cls " << angle_cls << std::endl;
            cv::rotate(textimg, textimg, cv::ROTATE_90_COUNTERCLOCKWISE);
        }
        std::string text = rec_model.predict_text(textimg);
        if(i != 0) os << "\n";
        os << text;
    }
    std::string str = os.str();
    if(str.empty()){
        return "";
    }
    char *res = new char[str.size() + 1];
    memcpy(res, str.data(), str.size());
    res[str.size()] = '\0';
    return res;
}

const char* get_build_info(){
    return cv::getBuildInformation().c_str();
}