extern crate bindgen;
extern crate cc;
extern crate pkg_config;
use bindgen::callbacks::MacroParsingBehavior;
use bindgen::callbacks::ParseCallbacks;
use std::env;
use std::path::PathBuf;
#[derive(Debug)]
struct MyParseCallbacks;
impl ParseCallbacks for MyParseCallbacks {
    fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
        if name.starts_with("FP_") {
            return MacroParsingBehavior::Ignore;
        }

        MacroParsingBehavior::Default
    }
}
fn add_lib(library: pkg_config::Library, include_paths: &mut Vec<String>) {
    for lib in library.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }
    for link_path in library.link_paths {
        println!("cargo:rustc-link-search={}", link_path.to_str().unwrap());
    }
    for include_path in library.include_paths {
        include_paths.push(include_path.to_str().unwrap().to_string());
    }
}
fn main() {
    let lib_dir = env::var("FFMPEG_LIB_DIR").ok();
    let mut include_dirs: Vec<String> = Vec::new();
    let include_dir = env::var("FFMPEG_INCLUDE_DIR").ok();

    if let (Some(lib_dir), Some(include_dir)) = (lib_dir, include_dir) {
        println!("cargo:rustc-link-search={}", lib_dir);
        println!("cargo:rustc-link-lib=avcodec");
        println!("cargo:rustc-link-lib=avformat");
        println!("cargo:rustc-link-lib=avutil");
        println!("cargo:rustc-link-lib=swscale");
        println!("cargo:rustc-link-lib=swresample");
        include_dirs.push(include_dir);
    } else {
        let lib = pkg_config::probe_library("libavcodec").unwrap();
        add_lib(lib, &mut include_dirs);
        let lib = pkg_config::probe_library("libavformat").unwrap();
        add_lib(lib, &mut include_dirs);
        let lib = pkg_config::probe_library("libavutil").unwrap();
        add_lib(lib, &mut include_dirs);
        let lib = pkg_config::probe_library("libswscale").unwrap();
        add_lib(lib, &mut include_dirs);
        let lib = pkg_config::probe_library("libswresample").unwrap();
        add_lib(lib, &mut include_dirs);
    };
    {
        let mut builder = cc::Build::new();
        for include_dir in include_dirs.iter() {
            builder.include(include_dir);
        }

        builder
            .file("c/averror.c")
            .flag_if_supported("-Wno-pointer-to-int-cast")
            .compile("ffmpeg_sys");
    }
    {
        let mut builder = bindgen::Builder::default().header("c/bindgen.h");
        for include_dir in include_dirs.iter() {
            builder = builder.clang_arg(format!("-I{}", &include_dir))
        }

        let bindings = builder
            .parse_callbacks(Box::new(MyParseCallbacks))
            .generate()
            .expect("Unable to generate bindings");
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
        // Write the bindings to the $OUT_DIR/bindings.rs file.
        bindings
            .write_to_file(out_path)
            .expect("Couldn't write bindings!");
    }
}
