//
// Created by mslxl on 5/7/2023.
//

#include "text_angle_cls.h"


TextClassifier::TextClassifier(const std::string &model_path) {
    std::wstring widestr(model_path.begin(), model_path.end());
    session_option.SetGraphOptimizationLevel(ORT_ENABLE_EXTENDED);
    ort_session = new Ort::Session(env, widestr.c_str(), session_option);

    auto num_input_nodes = ort_session->GetInputCount();
    auto num_output_nodes = ort_session->GetOutputCount();



    for(int i = 0; i < num_input_nodes; i++){
        auto name = ort_session->GetInputNameAllocated(i, allocator);

        input_names.emplace_back(name.release());

        Ort::TypeInfo input_type_info = ort_session->GetInputTypeInfo(i);
        auto input_tensor_info = input_type_info.GetTensorTypeAndShapeInfo();
        auto input_dims = input_tensor_info.GetShape();
        input_node_dims.push_back(input_dims);
    }

    for (int i = 0; i < num_output_nodes; i++) {
        auto name = ort_session->GetOutputNameAllocated(i, allocator);
        output_names.push_back(name.release());
        Ort::TypeInfo output_type_info = ort_session->GetOutputTypeInfo(i);
        auto output_tensor_info = output_type_info.GetTensorTypeAndShapeInfo();
        auto output_dims = output_tensor_info.GetShape();
        output_node_dims.push_back(output_dims);
    }
    num_out = output_node_dims[0][1];
}

TextClassifier::~TextClassifier() {
    for(auto ptr: input_names){
        allocator.Free(ptr);
    }
    for(auto ptr: output_names){
        allocator.Free(ptr);
    }
}

cv::Mat TextClassifier::preprocess(const cv::Mat& src) {
    cv::Mat dst;
    const int h = src.rows;
    const int w = src.cols;

    const double ration = double(w) /h;

    int resized_w = int(std::ceil(inp_height * ration));
    if(std::ceil(inp_height *  ration) > inp_width){
        resized_w = inp_width;
    }
    cv::resize(src, dst, cv::Size(resized_w, inp_height), cv::INTER_LINEAR);
    return dst;
}

void TextClassifier::normalize(cv::Mat img) {
    const int row = img.rows;
    const int col = img.cols;
    input_image.resize(this->inp_height * this->inp_width * img.channels());
    for(int c = 0; c < 3; c++){
        for(int i = 0; i < row; i++){
            for(int j = 0; j < inp_width; j++){
                if(j < col){
                    float pix = img.ptr<uchar>(i)[j * 3 + c];
                    this->input_image[c * row * inp_width + i * inp_width + j] = (pix / 255.0 - 0.5) / 0.5;
                }else{
                    this->input_image[c * row * inp_width + i * inp_width + j] = 0;
                }
            }
        }
    }
}

int TextClassifier::predict(cv::Mat image) {
    cv::Mat dst = this->preprocess(image);
    this->normalize(dst);

    std::array<int64_t, 4> input_shape{1, 3, inp_height, inp_width};
    auto allocator_info = Ort::MemoryInfo::CreateCpu(OrtDeviceAllocator, OrtMemTypeCPU);
    Ort::Value input_tensor = Ort::Value::CreateTensor<float>(allocator_info, input_image.data(), input_image.size(), input_shape.data(), input_shape.size());


    std::vector<Ort::Value> ort_outputs = ort_session->Run(Ort::RunOptions{nullptr},
                                                           &input_names[0], &input_tensor, 1,
                                                           output_names.data(), output_names.size());

    const float *pdata = ort_outputs[0].GetTensorMutableData<float>();

    int max_id = 0;
    float max_prob = -1;
    for(int i = 0; i < num_out; i++){
        if(pdata[i] > max_prob){
            max_prob = pdata[i];
            max_id = i;
        }
    }
    return max_id;
}

