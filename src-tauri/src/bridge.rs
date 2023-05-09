use encoding::all::GBK;
use encoding::{DecoderTrap, EncoderTrap, Encoding};
use std::ffi::{CStr, CString};
use std::path::PathBuf;

pub fn text_recongize(image_path: &str) -> Option<String> {
    let encoded_path = GBK.encode(image_path, EncoderTrap::Strict).ok()?;
    let bytes = unsafe {
        let path = CString::new(encoded_path).unwrap();
        let result = wrapper::ocr_image(path.as_ptr());
        CStr::from_ptr(result).to_bytes()
    };
    if PathBuf::from(image_path).exists() {
        GBK.decode(bytes, DecoderTrap::Strict).ok()
    } else {
        None
    }
}
pub fn get_opencv_build_info() -> String {
    let cstr = unsafe { CStr::from_ptr(wrapper::get_build_info()) };
    cstr.to_str().unwrap().to_owned()
}
mod wrapper {
    include!("../cpp/bridge.rs");
}

mod tests {
    use crate::bridge::get_opencv_build_info;

    #[test]
    fn test_ocr() {
        use super::text_recongize;
        println!(
            "Currently working in: {:?}",
            std::env::current_exe().unwrap()
        );
        let ans = text_recongize("cpp/test_image.jpg");
        println!("{:?}", ans);
    }

    #[test]
    fn text_build_info() {
        println!("{}", get_opencv_build_info());
    }
}
