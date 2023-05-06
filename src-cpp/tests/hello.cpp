#include <iostream>
#include <opencv2/opencv.hpp>
int main() {
    std::cout << "Hello, World!" << std::endl;
    auto icon = cv::imread("../../src-tauri/icons/icon.png");
    cv::imshow("Icon", icon);
    cv::waitKey();
    return 0;
}
