use std::fs;
use std::path::{Path, PathBuf};
use zip_archive::Format;
use zip_archive::Archiver;

// This file builds the project for windows.
// Usage: "cargo run --bin build-dir [target]"
// To build for windows, use "cargo run --bin build-dir x86_64-pc-windows-msvc"
// To build for x86 linux, use "cargo run --bin build-dir x86_64-unknown-linux-gnu"

fn main() {
    println!("Building");

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("Incorrect number of arguments ({}, need 2)", args.len());
    }

    std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg(format!("--target={}", args[1]))
        .stdout(std::io::stdout())
        .status()
        .expect("Failed to build!");

    println!("Moving files");

    fs::create_dir_all("./build").expect("Failed to create build dir");
    fs::copy("./README.md", "./build/README.md").expect("Failed to copy readme");
    copy_dir("./default", "./build/default").expect("Failed to copy default dir");
    copy_dir("./fonts", "./build/fonts").expect("Failed to copy default dir");
    copy_dir("./js", "./build/js").expect("Failed to copy js dir");
    copy_dir("./examples", "./build/examples").expect("Failed to copy js dir");

    // Try to move the executable file both on Windows and Linux and report error if both fails
    let windows_res = fs::copy(format!("./target/{}/release/cowtchoox.exe", args[1]), "./build/cowtchoox.exe");
    let other_res = fs::copy(format!("./target/{}/release/cowtchoox", args[1]), "./build/cowtchoox");
    if windows_res.is_err() && other_res.is_err() {
        println!("Failed to copy exe file. Consider moving it manually from `target/{}/release` to `build` folder", args[1])
    }

    let origin = PathBuf::from("./build");
    let dest = PathBuf::from("./");

    println!("Zipping");

    let mut archiver = Archiver::new();
    archiver.push(origin);
    archiver.set_destination(dest);
    archiver.set_format(Format::Zip);
    archiver.archive().expect("Failed to zip");

    println!("Finished: created build directory and build.zip");
}   

// From https://stackoverflow.com/questions/26958489/how-to-copy-a-folder-recursively-in-rust
fn copy_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

