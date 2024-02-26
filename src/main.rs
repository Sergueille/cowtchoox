#![allow(dead_code)]

mod parser;
mod writer;
mod doc_options;
mod browser;
mod log;
mod util;

use std::{collections::HashMap, fs, path::PathBuf};

use clap;
use parser::custom::{CustomTag, TagHash};

// Interpret command line arguments

pub struct Args {
    pub headful: bool,
    pub keep_alive: bool,
    pub filepath: String,
}

fn main() -> Result<(), ()> {
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
            .arg(
                clap::arg!(--cowx <FILE> "Includes a cowx file")
            )
            .get_matches();

    // Get the filepath from arguments
    let args = Args {
        filepath: matches.get_one::<String>("FILE").unwrap().clone(),
        headful: *matches.get_one::<bool>("headful").unwrap(),
        keep_alive: *matches.get_one::<bool>("keepalive").unwrap(),
    };

    let mut custom_tags_hash = HashMap::new(); // Store tags in this
    
    custom_tags_hash = parse_cowx_file("default/default.cowx", custom_tags_hash, &args, true)?; // FIXME: use the right path when not running with cargo 

    // Cowx file from command line
    let cowx_file = matches.get_one::<String>("cowx");
    match cowx_file {
        Some(file_name) => {
            match parse_cowx_file(file_name, custom_tags_hash, &args, false) {
                Ok(hash) => custom_tags_hash = hash,
                Err(_) => { return Ok(()); },
            }
        },
        None => { },
    }
    
    let res = std::fs::read_to_string(args.filepath.clone());

    match res {
        Ok(content) => {
            let mut path = std::env::current_dir().expect("Failed to get working dir");
            path.push(&args.filepath);

            compile_file(path, content, &args, custom_tags_hash);
        },
        Err(err) => {
            log::error(&format!("failed to read source file: {}", err));
        },
    }

    return Ok(());
}


fn compile_file(absolute_path: PathBuf, content: String, args: &Args, custom_tags_hash: TagHash) {
    let parser_context = parser::ParserContext {
        args,
        math_operators: custom_tags_hash,
        ignore_aliases: false,
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

    // Render to pdf!
    let _res = browser::render_to_pdf(out_path, args, &options);

    log::log("Done!");
}


pub fn parse_cowx_file(file_name: &str, custom_tags_hash: HashMap<String, CustomTag>, arguments: &Args, is_default: bool) -> Result<HashMap<String, CustomTag>, ()> {
    match std::fs::read_to_string(file_name) { // Try to read the file
        Ok(content) => {
            // Parse the file
            let res_hash = parser::custom::parse_custom_tags(
                &content.chars().collect::<Vec<char>>(), 
                &mut parser::get_start_of_file_position(PathBuf::from(file_name)), 
                custom_tags_hash, 
                &arguments,
                is_default
            );

            match res_hash {
                Ok(hash) => return Ok(hash),
                Err(err) => {
                    log::error_position(&err.message, &err.position, err.length);
                    return Err(()); // Fatal error, we're done!
                }
            }
        },
        Err(err) => {
            log::error(&format!("Failed to read cowx file at {}: {}", file_name, err));
            return Err(());
        }
    } 
}

