fn main() {
    //println!("cargo:rustc-link-lib=dylib=libdds");
    //println!("cargo:rustc-link-search=native=/home/alvaro/Programming/workspace-rust/rusty-dealer/libdds.so");
    println!(
        "cargo:rustc-link-arg=-Wl,/home/alvaro/Programming/workspace-rust/rusty-dealer/libdds.so"
    );
}
