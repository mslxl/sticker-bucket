fn main() {
    lalrpop::process_root().unwrap();
    tauri_build::build()
}
