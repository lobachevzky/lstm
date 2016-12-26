use std::process::Command;
use std::{env, fs, str};
use std::path::Path;

fn more_recent_than(srcs: &Vec<String>, dst: &str) -> std::io::Result<bool> {
    match fs::metadata(dst) {
        Ok(metadata_dst) => {
            let time_mod_dst = metadata_dst.modified()?;
            for src in srcs {
                let time_mod_src = fs::metadata(src)?.modified()?;

                if time_mod_src > time_mod_dst {
                    return Ok(true)
                }
            } 
            return Ok(false)
        }
        _ => Ok(true)
    }
}

fn main() {
    let ext;
    let compiler;
    let flags;
    let dir;
    if Command::new("nvcc").status().is_ok() {
        ext = "cu";
        compiler = "nvcc";
        flags = "-lcublas";
        dir = "src/gpu";
        println!("cargo:rustc-link-lib=dylib=cublas");
        println!("cargo:rustc-link-lib=dylib=cudart");
    } else {
        ext = "cpp";
        compiler = "cc";
        flags = "";
        dir = "src/cpu";
    };
    let c_names = vec!["matrix", "ops", "util"];

    let out_dir = env::var("OUT_DIR").unwrap();
    let get_out_name = |name| format!("{}/{}.o", out_dir, name);

    for i in 0..c_names.len() {
        let src_name = format!("{}/{}.{}", dir, c_names[i], ext);
        let out_name = get_out_name(c_names[i]);

        if more_recent_than(&vec![src_name.clone()], &out_name).unwrap() {
            assert!(Command::new(compiler)
                .arg(&src_name)
                .args(&["-c", "-Xcompiler", "-fPIC", flags, "-o"]) 
                .arg(&out_name)
                .status().unwrap().success(), "{} {} failed", compiler, src_name);
        }
    }

    let out_files: Vec<String> = c_names.into_iter().map(get_out_name).collect();

    if more_recent_than(&out_files, "libmatrix.a").unwrap() {

        assert!(Command::new("rm")
            .args(&["-f", "libmatrix.a"]) 
            .current_dir(&Path::new(&out_dir)) 
            .status().unwrap().success(), "rm failed");


        assert!(Command::new("ar")
            .args(&["crus", "libmatrix.a"])
            .args(&out_files)
            .current_dir(&Path::new(&out_dir)) 
            .status().unwrap().success(), "ar failed");
    }

    println!("cargo:rustc-link-search=native={}", out_dir);
    if let Some(paths) = env::var_os("LD_LIBRARY_PATH") {
        for path in env::split_paths(&paths) {
            println!("cargo:rustc-link-search=native={}", path.display());
        }
    }
    println!("cargo:rustc-link-lib=static=matrix");
}
