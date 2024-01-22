
mod parser;
mod writer;
mod doc_options;
mod browser;

use std::{path::PathBuf, fs};

use clap;

// Interpret command line arguments

fn main() {
    let matches =
        clap::Command::new("cowtchoox") 
            .arg_required_else_help(true)
            .arg(
                clap::arg!(<FILE> "Path to the file to compile")
            )
            .get_matches();

    // Get the filepath from arguments
    let filepath = matches.get_one::<String>("FILE").unwrap();
    
    let res = std::fs::read_to_string(filepath);

    match res {
        Ok(content) => {
            let mut path = std::env::current_dir().expect("Failed to get working dir");
            path.push(&filepath);

            compile_file(path, content);
        },
        Err(err) => {
            println!("ERR: failed to read source file: {}", err);
        },
    }

    println!("Finished!");
}

fn compile_file(absolute_path: PathBuf, content: String) {
    // TEST: testing parser here

    let document = parser::parse_file(&absolute_path, &content.chars().collect()).expect("Failed to parse file");
    let text = writer::get_file_text(&document).expect("Failed to create HTML");

    println!("HTML output: \n{}", text);

    // TODO: make this in a mor elegant way
    let mut out_path = std::env::current_dir().expect("Failed to get working dir");
    out_path.push("out.html");
    fs::write(out_path.clone(), text).unwrap();

    // Render to pdf!
    browser::render_to_pdf(out_path);
}

