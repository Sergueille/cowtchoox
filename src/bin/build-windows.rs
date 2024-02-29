use std::fs;
use std::path::{Path, PathBuf};
use zip_archive::Format;
use zip_archive::Archiver;

// This file builds the project for windows.
// Call it with "cargo run --bin build-windows"

fn main() {
    println!("Building");

    std::process::Command::new("cargo").arg("build").arg("--release").stdout(std::process::Stdio::null()).output().expect("Failed to build!");

    println!("Moving files");

    fs::create_dir_all("./build").expect("Failed to create build dir");
    fs::copy("./target/release/cowtchoox.exe", "./build/cowtchoox.exe").expect("Failed to copy exe");
    fs::copy("./README.md", "./build/README.md").expect("Failed to copy readme");
    copy_dir("./default", "./build/default").expect("Failed to copy default dir");
    copy_dir("./fonts", "./build/fonts").expect("Failed to copy default dir");
    copy_dir("./js", "./build/js").expect("Failed to copy js dir");
    copy_dir("./examples", "./build/examples").expect("Failed to copy js dir");

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

