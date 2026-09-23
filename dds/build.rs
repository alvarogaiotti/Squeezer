// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

// Compiles the vendored DDS 3.x solver into a static archive.
//
// Upstream ships a Bazel-only build, so the core translation units under
// `vendor/dds/library/src` are compiled with a hand-rolled C++ invocation and
// archived with `ar`. The resulting archive is linked into every crate that
// depends on `dds` through the `cargo:rustc-link-*` directives below, which is
// why this crate owns `links = "dds"` rather than the root package.
//
// TODO: this targets Linux/ELF (GCC flags, `ar`, `stdc++`/`pthread`/`m`); a
// Windows/MSVC build would need its own flags and library naming.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SUBMODULE_HINT: &str =
    "run `git submodule update --init vendor/dds` to fetch the vendored DDS source";

fn collect_cpp(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}\n{SUBMODULE_HINT}", dir.display()));
    for entry in entries {
        let entry = entry.expect("directory entries are readable");
        let path = entry.path();
        if path.is_dir() {
            collect_cpp(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "cpp") {
            out.push(path);
        }
    }
}

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cargo sets manifest dir"));
    let dds_src = manifest
        .parent()
        .expect("dds crate is nested under the repo root")
        .join("vendor/dds/library/src");
    if !dds_src.is_dir() {
        panic!(
            "vendored DDS source not found at {}\n{SUBMODULE_HINT}",
            dds_src.display()
        );
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("cargo sets out dir"));
    let obj_dir = out_dir.join("obj");
    fs::create_dir_all(&obj_dir).expect("create object directory");

    let mut sources = Vec::new();
    collect_cpp(&dds_src, &mut sources);
    sources.sort();

    let cxx = env::var("CXX").unwrap_or_else(|_| "g++".into());
    let include = format!("-I{}", dds_src.display());

    let mut objects = Vec::new();
    for src in &sources {
        // Flatten the path relative to the DDS source root into the object
        // name so same-named translation units in different subdirectories
        // cannot overwrite each other.
        let rel = src.strip_prefix(&dds_src).expect("source under DDS src");
        let obj_name = rel.to_string_lossy().replace(['/', '\\'], "_");
        let obj = obj_dir.join(format!("{obj_name}.o"));
        let status = Command::new(&cxx)
            .args(["-std=c++20", "-O3", "-fPIC", "-w"])
            .arg(&include)
            .arg("-c")
            .arg(src)
            .arg("-o")
            .arg(&obj)
            .status()
            .unwrap_or_else(|e| panic!("failed to spawn {cxx}: {e}"));
        assert!(status.success(), "compiling {} failed", src.display());
        objects.push(obj);
    }

    let archive = out_dir.join("libdds3.a");
    let _ = fs::remove_file(&archive);
    let status = Command::new("ar")
        .arg("crs")
        .arg(&archive)
        .args(&objects)
        .status()
        .expect("spawn ar");
    assert!(status.success(), "archiving libdds3.a failed");

    println!("cargo:rerun-if-changed={}", dds_src.display());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=dds3");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=pthread");
    println!("cargo:rustc-link-lib=dylib=m");
}
