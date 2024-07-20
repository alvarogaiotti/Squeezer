// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

fn main() {
    let path = std::env::current_dir().unwrap();
    let path = path.join("libdds.so");
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,{}", path.display());

    #[cfg(target_os = "windows")]
    println!(
        "cargo:rustc-link-arg=C:\\Users\\gttlv\\RustProjects\\bridge_bro\\squeezer\\dds\\dds.lib"
    );
}
