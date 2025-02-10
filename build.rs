use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let cargo_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let lib_path = format!("{}\\SDL2", cargo_path);
    let dll_path = format!("{}\\SDL2\\SDL2.dll", cargo_path);
    let dll_destination = format!("{}\\target\\debug\\SDL2.dll", cargo_path);

    println!("cargo::rustc-link-lib=static=SDL2main");
    println!("cargo::rustc-link-lib=static=SDL2");

    println!("cargo::rustc-link-search=native={}", lib_path);

    if let Err(e) = fs::copy(dll_path, dll_destination) {
        panic!("Failed to copy SDL2.dll: {}", e);
    }
}
