//
// Created by mslxl on 5/8/2023.
//

#include "text_det.h"


TextDetector::TextDetector(const std::string &model_path) {
    this->binary_threshold = 0.3;
    this->polygon_threshold = 0.5;
    this->unclip_ratio = 1.6;
    this->max_candidates = 1000;

    std::wstring widestr(model_path.begin(), model_path.end());

    session_options.SetGraphOptimizationLevel(ORT_ENABLE_EXTENDED);
    this->net = new Ort::Session(env, widestr.c_str(), session_options);
    auto num_input_nodes = net->GetInputCount();
    auto num_output_nodes = net->GetOutputCount();


    for (int i = 0; i < num_input_nodes; i++) {
        auto name = net->GetInputNameAllocated(i, allocator);

        input_names.emplace_back(name.release());

    }

    for (int i = 0; i < num_output_nodes; i++) {
        auto name = net->GetOutputNameAllocated(i, allocator);
        output_names.push_back(name.release());

    }
}

cv::Mat TextDetector::preprocess(const cv::Mat &src) {
    cv::Mat dst;
    cvtColor(src, dst, cv::COLOR_BGR2RGB);
    int h = src.rows;
    int w = src.cols;
    float scale_h = 1;
    float scale_w = 1;
    if (h < w) {
        scale_h = (float) this->short_size / (float) h;
        float tar_w = (float) w * scale_h;
        tar_w = tar_w - (int) tar_w % 32;
        tar_w = std::max((float) 32, tar_w);
        scale_w = tar_w / (float) w;
    } else {
        scale_w = (float) this->short_size / (float) w;
        float tar_h = (float) h * scale_w;
        tar_h = tar_h - (int) tar_h % 32;
        tar_h = std::max((float) 32, tar_h);
        scale_h = tar_h / (float) h;
    }
    resize(dst, dst, cv::Size(int(scale_w * dst.cols), int(scale_h * dst.rows)), cv::INTER_LINEAR);
    return dst;
}

void TextDetector::normalize(cv::Mat img) {
    int row = img.rows;
    int col = img.cols;
    this->input_image.resize(row * col * img.channels());
    for (int c = 0; c < 3; c++) {
        for (int i = 0; i < row; i++) {
            for (int j = 0; j < col; j++) {
                float pix = img.ptr<uchar>(i)[j * 3 + c];
                this->input_image[c * row * col + i * col + j] =
                        (pix / 255.0 - this->mean_values[c]) / this->norm_values[c];
            }
        }
    }
}

std::vector<std::vector<cv::Point2f> > TextDetector::detect(cv::Mat &srcimg) {
    int h = srcimg.rows;
    int w = srcimg.cols;
    cv::Mat dstimg = this->preprocess(srcimg);
    this->normalize(dstimg);
    std::array<int64_t, 4> input_shape{1, 3, dstimg.rows, dstimg.cols};

    auto allocator_info = Ort::MemoryInfo::CreateCpu(OrtDeviceAllocator, OrtMemTypeCPU);
    Ort::Value input_tensor_ = Ort::Value::CreateTensor<float>(allocator_info, input_image.data(), input_image.size(),
                                                               input_shape.data(), input_shape.size());

    std::vector<Ort::Value> ort_outputs = net->Run(Ort::RunOptions{nullptr}, &input_names[0], &input_tensor_, 1,
                                                   output_names.data(), output_names.size());
    const float *floatArray = ort_outputs[0].GetTensorMutableData<float>();
    
    auto outputInfo = ort_outputs[0].GetTensorTypeAndShapeInfo();

    int outputCount = 1;
    for (unsigned int shapeI = 0; shapeI < outputInfo.GetShape().size(); shapeI++){
        outputCount *= outputInfo.GetShape()[shapeI];
    }

    cv::Mat binary(dstimg.rows, dstimg.cols, CV_32FC1);
    memcpy(binary.data, floatArray, std::min(outputCount, dstimg.rows * dstimg.cols) * sizeof(float));

    // Threshold
    cv::Mat bitmap;
    threshold(binary, bitmap, binary_threshold, 255, cv::THRESH_BINARY);
    // Scale ratio
    float scaleHeight = (float) (h) / (float) (binary.size[0]);
    float scaleWidth = (float) (w) / (float) (binary.size[1]);
    // Find contours
    std::vector<std::vector<cv::Point> > contours;
    bitmap.convertTo(bitmap, CV_8UC1);
    findContours(bitmap, contours, cv::RETR_LIST, cv::CHAIN_APPROX_SIMPLE);

    // Candidate number limitation
    size_t numCandidate = std::min(contours.size(), (size_t) (max_candidates > 0 ? max_candidates : INT_MAX));
    std::vector<float> confidences;
    std::vector<std::vector<cv::Point2f> > results;
    for (size_t i = 0; i < numCandidate; i++) {
        std::vector<cv::Point> &contour = contours[i];

        // Calculate text contour score
        if (contour_score(binary, contour) < polygon_threshold)
            continue;

        // Rescale
        std::vector<cv::Point> contourScaled;
        contourScaled.reserve(contour.size());
        for (size_t j = 0; j < contour.size(); j++) {
            contourScaled.push_back(cv::Point(int(contour[j].x * scaleWidth),
                                              int(contour[j].y * scaleHeight)));
        }

        // Unclip
        cv::RotatedRect box = minAreaRect(contourScaled);
        float longSide = std::max(box.size.width, box.size.height);
        if (longSide < long_side_thresh) {
            continue;
        }

        // minArea() rect is not normalized, it may return rectangles with angle=-90 or height < width
        const float angle_threshold = 60;  // do not expect vertical text, TODO detection algo property
        bool swap_size = false;
        if (box.size.width < box.size.height)  // horizontal-wide text area is expected
            swap_size = true;
        else if (fabs(box.angle) >= angle_threshold)  // don't work with vertical rectangles
            swap_size = true;
        if (swap_size) {
            std::swap(box.size.width, box.size.height);
            if (box.angle < 0)
                box.angle += 90;
            else if (box.angle > 0)
                box.angle -= 90;
        }

        cv::Point2f vertex[4];
        box.points(vertex);  // order: bl, tl, tr, br

        std::vector<cv::Point2f> approx;
        for (int j = 0; j < 4; j++)
            approx.emplace_back(vertex[j]);
        std::vector<cv::Point2f> polygon;
        unclip(approx, polygon);

        box = minAreaRect(polygon);
        longSide = std::max(box.size.width, box.size.height);
        if (longSide < long_side_thresh + 2) {
            continue;
        }
        for(auto& v: polygon){
            v.x = std::min(v.x, float(srcimg.cols-1));
            v.y = std::min(v.y, float(srcimg.rows-1));
            v.x = std::max(v.x, float(0));
            v.y = std::max(v.y, float(0));
        }

        results.push_back(polygon);
    }
    confidences = std::vector<float>(contours.size(), 1.0f);
    return results;
}

std::vector<std::vector<cv::Point2f> >
TextDetector::order_points_clockwise(std::vector<std::vector<cv::Point2f> > results) {
    std::vector<std::vector<cv::Point2f> > order_points(results);
    for (int i = 0; i < results.size(); i++) {
        float max_sum_pts = -10000;
        float min_sum_pts = 10000;
        float max_diff_pts = -10000;
        float min_diff_pts = 10000;

        int max_sum_pts_id = 0;
        int min_sum_pts_id = 0;
        int max_diff_pts_id = 0;
        int min_diff_pts_id = 0;
        for (int j = 0; j < 4; j++) {
            const float sum_pt = results[i][j].x + results[i][j].y;
            if (sum_pt > max_sum_pts) {
                max_sum_pts = sum_pt;
                max_sum_pts_id = j;
            }
            if (sum_pt < min_sum_pts) {
                min_sum_pts = sum_pt;
                min_sum_pts_id = j;
            }

            const float diff_pt = results[i][j].y - results[i][j].x;
            if (diff_pt > max_diff_pts) {
                max_diff_pts = diff_pt;
                max_diff_pts_id = j;
            }
            if (diff_pt < min_diff_pts) {
                min_diff_pts = diff_pt;
                min_diff_pts_id = j;
            }
        }
        order_points[i][0].x = results[i][min_sum_pts_id].x;
        order_points[i][0].y = results[i][min_sum_pts_id].y;
        order_points[i][2].x = results[i][max_sum_pts_id].x;
        order_points[i][2].y = results[i][max_sum_pts_id].y;

        order_points[i][1].x = results[i][min_diff_pts_id].x;
        order_points[i][1].y = results[i][min_diff_pts_id].y;
        order_points[i][3].x = results[i][max_diff_pts_id].x;
        order_points[i][3].y = results[i][max_diff_pts_id].y;
    }
    return order_points;
}

void TextDetector::draw_pred(cv::Mat &srcimg, std::vector<std::vector<cv::Point2f> > results) {
    for (int i = 0; i < results.size(); i++) {
        for (int j = 0; j < 4; j++) {
            circle(srcimg, cv::Point((int) results[i][j].x, (int) results[i][j].y), 2, cv::Scalar(0, 0, 255), -1);
            if (j < 3) {
                line(srcimg, cv::Point((int) results[i][j].x, (int) results[i][j].y),
                     cv::Point((int) results[i][j + 1].x, (int) results[i][j + 1].y), cv::Scalar(0, 255, 0));
            } else {
                line(srcimg, cv::Point((int) results[i][j].x, (int) results[i][j].y),
                     cv::Point((int) results[i][0].x, (int) results[i][0].y), cv::Scalar(0, 255, 0));
            }
        }
    }
}

float TextDetector::contour_score(const cv::Mat &binary, const std::vector<cv::Point> &contour) {
    cv::Rect rect = boundingRect(contour);
    int xmin = std::max(rect.x, 0);
    int xmax = std::min(rect.x + rect.width, binary.cols - 1);
    int ymin = std::max(rect.y, 0);
    int ymax = std::min(rect.y + rect.height, binary.rows - 1);

    cv::Mat binROI = binary(cv::Rect(xmin, ymin, xmax - xmin + 1, ymax - ymin + 1));

    cv::Mat mask = cv::Mat::zeros(ymax - ymin + 1, xmax - xmin + 1, CV_8U);
    std::vector<cv::Point> roiContour;
    for (size_t i = 0; i < contour.size(); i++) {
        cv::Point pt = cv::Point(contour[i].x - xmin, contour[i].y - ymin);
        roiContour.push_back(pt);
    }
    std::vector<std::vector<cv::Point>> roiContours = {roiContour};
    fillPoly(mask, roiContours, cv::Scalar(1));
    float score = mean(binROI, mask).val[0];
    return score;
}

void TextDetector::unclip(const std::vector<cv::Point2f> &inPoly, std::vector<cv::Point2f> &outPoly) {
    float area = contourArea(inPoly);
    float length = arcLength(inPoly, true);
    float distance = area * unclip_ratio / length;

    size_t numPoints = inPoly.size();
    std::vector<std::vector<cv::Point2f>> newLines;
    for (size_t i = 0; i < numPoints; i++) {
        std::vector<cv::Point2f> newLine;
        cv::Point pt1 = inPoly[i];
        cv::Point pt2 = inPoly[(i - 1) % numPoints];
        cv::Point vec = pt1 - pt2;
        float unclipDis = (float) (distance / norm(vec));
        cv::Point2f rotateVec = cv::Point2f(vec.y * unclipDis, -vec.x * unclipDis);
        newLine.push_back(cv::Point2f(pt1.x + rotateVec.x, pt1.y + rotateVec.y));
        newLine.push_back(cv::Point2f(pt2.x + rotateVec.x, pt2.y + rotateVec.y));
        newLines.push_back(newLine);
    }

    size_t numLines = newLines.size();
    for (size_t i = 0; i < numLines; i++) {
        cv::Point2f a = newLines[i][0];
        cv::Point2f b = newLines[i][1];
        cv::Point2f c = newLines[(i + 1) % numLines][0];
        cv::Point2f d = newLines[(i + 1) % numLines][1];
        cv::Point2f pt;
        cv::Point2f v1 = b - a;
        cv::Point2f v2 = d - c;
        float cosAngle = (v1.x * v2.x + v1.y * v2.y) / (norm(v1) * norm(v2));

        if (fabs(cosAngle) > 0.7) {
            pt.x = (b.x + c.x) * 0.5;
            pt.y = (b.y + c.y) * 0.5;
        } else {
            float denom = a.x * (float) (d.y - c.y) + b.x * (float) (c.y - d.y) +
                          d.x * (float) (b.y - a.y) + c.x * (float) (a.y - b.y);
            float num = a.x * (float) (d.y - c.y) + c.x * (float) (a.y - d.y) + d.x * (float) (c.y - a.y);
            float s = num / denom;

            pt.x = a.x + s * (b.x - a.x);
            pt.y = a.y + s * (b.y - a.y);
        }
        outPoly.push_back(pt);
    }
}

cv::Mat TextDetector::get_rotate_crop_image(const cv::Mat &frame, std::vector<cv::Point2f> vertices) {
    cv::Rect rect = boundingRect(cv::Mat(vertices));
    cv::Mat crop_img = frame(rect);

    const cv::Size outputSize = cv::Size(rect.width, rect.height);

    std::vector<cv::Point2f> targetVertices{cv::Point2f(0, outputSize.height), cv::Point2f(0, 0),
                                            cv::Point2f(outputSize.width, 0),
                                            cv::Point2f(outputSize.width, outputSize.height)};
    //vector<Point2f> targetVertices{ Point2f(0, 0), Point2f(outputSize.width, 0), Point2f(outputSize.width, outputSize.height), Point2f(0, outputSize.height) };

    for (int i = 0; i < 4; i++) {
        vertices[i].x -= rect.x;
        vertices[i].y -= rect.y;
    }

    cv::Mat rotationMatrix = getPerspectiveTransform(vertices, targetVertices);
    cv::Mat result;
    warpPerspective(crop_img, result, rotationMatrix, outputSize, cv::BORDER_REPLICATE);
    return result;
}

TextDetector::~TextDetector() {
    for(auto ptr: input_names){
        allocator.Free(ptr);
    }
    for(auto ptr: output_names){
        allocator.Free(ptr);
    }
}
