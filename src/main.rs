#![allow(dead_code)]

mod parser;
mod writer;
mod doc_options;
mod browser;
mod log;
mod util;

use std::{collections::HashMap, fs, path::PathBuf};

use clap;

// Interpret command line arguments

pub struct Args {
    pub headful: bool,
    pub keep_alive: bool,
    pub filepath: String,
}

fn main() {
    let matches =
        clap::Command::new("cowtchoox") 
            .arg_required_else_help(true)
            .arg(
                clap::arg!(<FILE> "Path to the file to compile")
            )
            .arg(
                clap::arg!(--headful "Actually opens the browser window")
            )
            .arg(
                clap::arg!(--keepalive "Keeps the browser opened until the program is forced to stop")
            )
            .get_matches();

    // Get the filepath from arguments
    let args = Args {
        filepath: matches.get_one::<String>("FILE").unwrap().clone(),
        headful: *matches.get_one::<bool>("headful").unwrap(),
        keep_alive: *matches.get_one::<bool>("keepalive").unwrap(),
    };
    
    let res = std::fs::read_to_string(args.filepath.clone());

    match res {
        Ok(content) => {
            let mut path = std::env::current_dir().expect("Failed to get working dir");
            path.push(&args.filepath);

            compile_file(path, content, &args);
        },
        Err(err) => {
            log::error(&format!("failed to read source file: {}", err));
        },
    }
}

fn compile_file(absolute_path: PathBuf, content: String, args: &Args) {
    // TEST: testing parser here

    let parser_context = parser::ParserContext {
        args,
        math_operators: HashMap::new(),
    };

    log::log("Parsing document...");
    let document = match parser::parse_file(&absolute_path, &content.chars().collect(), &parser_context) {
        Ok(node) => node,
        Err(err) => {
            log::error_position(&err.message, &err.position, err.length);
            return;
        },
    };
    
    log::log("Creating HTML...");
    let (text, options) = writer::get_file_text(&document).expect("Failed to create HTML");

    // TODO: make this in a mor elegant way
    let mut out_path = std::env::current_dir().expect("Failed to get working dir");
    out_path.push("out.html");
    fs::write(out_path.clone(), text).unwrap();

    log::log("Creating PDF...");

    // Render to pdf!
    let _res = browser::render_to_pdf(out_path, args, &options);

    log::log("Done!");
}

