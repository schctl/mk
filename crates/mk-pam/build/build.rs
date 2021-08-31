fn main() {
    println!("cargo:rustc-link-lib=pam");
    println!("cargo:rerun-if-changed=build/wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("build/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("ffi.rs"))
        .expect("Couldn't write bindings!");
}
