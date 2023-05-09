//
// Created by mslxl on 5/7/2023.
//

#ifndef SRC_CPP_BRIDGE_H
#define SRC_CPP_BRIDGE_H


#ifdef __cplusplus
extern "C" {
#endif
    const char* ocr_image(const char* image_path);

    const char* get_build_info();
#ifdef __cplusplus
}
#endif

#endif //SRC_CPP_BRIDGE_H
