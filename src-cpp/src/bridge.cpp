//
// Created by mslxl on 5/7/2023.
//

#include "bridge.h"
#include "ocr/text_rec.h"
#include "ocr/text_det.h"
#include "ocr/text_angle_cls.h"


char* ocr_image(const char* image_path){
    TextDetector detect_model("model/ch_PP-OCRv3_det_infer.onnx");
    TextClassifier angle_model("model/ch_ppocr_mobile_v2.0_cls.onnx");
    TextRecognizer rec_model("model/ch_PP-OCRv3_rec_infer.onnx", "model/rec_word_dict.txt");

    cv::Mat src = cv::imread(image_path);

    std::vector<std::vector<cv::Point2f>> result = detect_model.detect(src);
    std::reverse(result.begin(), result.end());
    std::ostringstream os;
    for(std::size_t i = 0; i < result.size(); i++){
        cv::Mat textimg = detect_model.get_rotate_crop_image(src, result[i]);

        if(angle_model.predict(textimg)){
            cv::rotate(textimg, textimg, 1);
        }
        std::string text = rec_model.predict_text(textimg);
        if(i != 0) os << "\n";
        os << text;
    }
    std::string str = os.str();
    char *res = new char[str.size() + 1];
    memcpy(res, str.data(), str.size());
    res[str.size()] = '\0';
    return res;
}