fn main() {
  tauri_build::build();

  let bindings = bindgen::Builder::default()
    .header("../src-cpp/src/bridge.h")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .generate()
    .expect("Unable to generate bindings");

  bindings.write_to_file("libs/bridge.rs").expect("Couldn't write bindings!");
  println!("cargo:rustc-link-lib=mmm");
  println!("cargo:rustc-link-lib=opencv_core4");
  println!("cargo:rustc-link-lib=opencv_imgcodecs4");
  println!("cargo:rustc-link-lib=opencv_imgproc4");
  println!("cargo:rustc-link-lib=opencv_highgui4");
  println!("cargo:rustc-link-lib=onnxruntime");
  println!("cargo:rustc-link-search=libs/");
  println!("cargo:rerun-if-changed=../src-cpp/src/bridge.h");
  println!("cargo:rustc-link-arg-bins=-Wl,-rpath=$ORIGIN");
}
