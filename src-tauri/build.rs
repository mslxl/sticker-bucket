use std::env;

fn main() {
  tauri_build::build();

  let bindings = bindgen::Builder::default()
    .header("cpp/src/bridge.h")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .generate()
    .expect("Unable to generate bindings");

  bindings.write_to_file("cpp/bridge.rs").expect("Couldn't write bindings!");
  println!("cargo:rerun-if-changed=cpp/src/bridge.h");

  env::set_var("VCPKGRS_DYNAMIC", "1");
  vcpkg::Config::new()
    .target_triplet("x64-windows")
    .find_package("opencv")
    .unwrap();

  println!("cargo:rustc-link-search=cpp/build/");
  println!("cargo:rustc-link-lib=static=mmm");
  println!("cargo:rustc-link-search=cpp/3rd_party/ort/lib");
  println!("cargo:rustc-link-lib=dylib=onnxruntime");
  println!("cargo:rustc-link-arg-bins=-Wl,-rpath=$ORIGIN");
}
