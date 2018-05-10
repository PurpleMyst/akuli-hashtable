extern crate bindgen;
extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/hashtable.c")
        .compile("libhashtable.a");

    let bindings = bindgen::Builder::default()
        .blacklist_type("max_align_t")
        .trust_clang_mangling(false)
        .header("src/hashtable.h")
        .header("src/common.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
