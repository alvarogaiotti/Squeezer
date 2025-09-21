// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use std::{env, fs};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    #[cfg(target_os = "linux")]
    {
        fs::copy("libdds.so", format!("{}/libdds.so", out_dir)).unwrap();
        println!("cargo:rustc-link-search=native={}", out_dir);
        println!("cargo:rustc-link-lib=dylib=dds");
        //println!("cargo:rustc-link-arg=-Wl,{}", path.display());
    }
    //{
    //    let out_dir = env::var("OUT_DIR").unwrap();
    //    fs::copy("libdds.a", format!("{}/libdds.a", out_dir)).unwrap();
    //    println!("cargo:rustc-link-search=native={}", out_dir);
    //    println!("cargo:rustc-link-lib=static=dds");
    //    println!("cargo:rustc-link-lib=stdc++");
    //    println!("cargo:rustc-link-lib=gomp");
    //}
    #[cfg(target_os = "windows")]
    println!(
        "cargo:rustc-link-arg=C:\\Users\\gttlv\\RustProjects\\bridge_bro\\squeezer\\dds\\dds.lib"
    );
}
