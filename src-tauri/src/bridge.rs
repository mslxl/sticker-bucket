use std::ffi::{CString, CStr};
use encoding::{Encoding, DecoderTrap};
use encoding::all::GBK;

fn ocr_image(image_path: &str)->String{
   let bytes = unsafe{
    let path = CString::new(image_path).unwrap();
    let result = wrapper::ocr_image(path.as_ptr());
    CStr::from_ptr(result).to_bytes()
  };
  GBK.decode(bytes, DecoderTrap::Strict).unwrap()
}
mod wrapper{
  include!("../libs/bridge.rs");
}

mod tests{

  #[test]
  fn test_ocr(){
    use super::ocr_image;
    println!("Currently working in: {:?}", std::env::current_exe().unwrap());
    let ans = ocr_image("C:/Users/lnslf/Documents/sources/mslxl/meme-management/meme-management/src-cpp/test_image.jpg");
    println!("{}", ans);
  }
}