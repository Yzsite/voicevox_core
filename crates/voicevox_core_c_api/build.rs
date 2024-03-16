fn main() {
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/");
        println!("cargo:rustc-link-arg=-Wl,-install_name,@rpath/libvoicevox_core.dylib");
    }

    if std::env::var("TARGET").unwrap() == "wasm32-unknown-emscripten" {
        use regex::Regex;
        use std::fs;
        // println!("cargo:rustc-link-arg=-sERROR_ON_UNDEFINED_SYMBOLS=0");
        // // TODO: WARNにしたいけど、これにするとemccがクラッシュする（どうして...）
        // // println!("cargo:rustc-link-arg=-sWARN_ON_UNDEFINED_SYMBOLS=1");
        // println!(
        //     "cargo:rustc-link-arg=-sEXPORTED_FUNCTIONS=['{}']",
        //     functions.join("','")
        // );
        // println!("cargo:rustc-link-arg=-sEXPORTED_RUNTIME_METHODS=['ccall']");
        // println!("cargo:rustc-link-arg=-sEXPORT_NAME=\"RawVoicevoxCore\"");
        // println!("cargo:rustc-link-arg=-sMODULARIZE=1");
        // println!("cargo:rustc-link-arg=-sTOTAL_STACK=128MB");
        // println!("cargo:rustc-link-arg=-sINITIAL_MEMORY=256MB");
        println!("cargo:rustc-link-arg=-sALLOW_MEMORY_GROWTH=1");
        println!(
            "cargo:rustc-link-arg=--js-library={}",
            std::env::var("CARGO_MANIFEST_DIR").unwrap() + "/wasm_library.js"
        );

        let re = Regex::new(r#"pub (?:unsafe )?extern "C" fn (\w+)"#).unwrap();
        let mut functions = vec![
            "_malloc".to_string(),
            "_free".to_string(),
            "_setenv".to_string(),
        ];
        let lib_rs = fs::read_to_string("src/lib.rs").unwrap();
        let wasm_rs = fs::read_to_string("src/wasm.rs").unwrap();
        for cap in re.captures_iter(&lib_rs) {
            functions.push(format!("_{}", cap[1].to_string()));
        }
        for cap in re.captures_iter(&wasm_rs) {
            functions.push(format!("_{}", cap[1].to_string()));
        }
        println!(
            "cargo:rustc-link-arg=-sEXPORTED_RUNTIME_METHODS=[\"{}\"]",
            [
                "ccall",
                "dynCall",
                "stackSave",
                "stackRestore",
                "stackAlloc",
                "stackFree"
            ]
            .join("\",\"")
        );
        println!(
            "cargo:rustc-link-arg=-sEXPORTED_FUNCTIONS=['{}']",
            functions.join("','")
        );
        // TODO: ちゃんと絞る
        println!("cargo:rustc-link-arg=-sEXPORT_ALL=1");

        // println!("cargo:rustc-link-arg=--no-entry");
        let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
        let target_dir = out_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        println!(
            "cargo:rustc-link-arg=-o{}/voicevox_core_wasm_api.mjs",
            target_dir.display()
        );
        println!("cargo:rustc-link-arg=-sERROR_ON_UNDEFINED_SYMBOLS=0");
        println!("cargo:rustc-link-arg=-sEXPORT_NAME=VoicevoxCore");
        println!("cargo:rustc-link-arg=-DEMSCRIPTEN_STANDALONE_WASM");
        println!("cargo:rustc-link-arg=--no-entry");
    }
}
