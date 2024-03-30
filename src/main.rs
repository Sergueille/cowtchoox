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

// This file interprets command line arguments, and call the different modules's functions

pub struct Args {
    pub headful: bool,
    pub keep_alive: bool,
    pub filepath: String,
    pub no_pdf: bool,
}


/// Contains useful information to parse a document
pub struct Context<'a> {
    pub args: &'a crate::Args, // Command line arguments
    pub custom_tags: TagHash,
    pub ignore_aliases: bool,
    pub default_dir: &'a PathBuf
}


fn main() -> Result<(), ()> {
    log::override_panic_message();

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
                clap::arg!(--"no-pdf" "Create no pdf file")
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
        no_pdf: *matches.get_one::<bool>("no-pdf").unwrap(),
    };

    let mut custom_tags_hash = HashMap::new(); // Store tags in this

    // HACK: assume cargo tests are always debug mode, and builds always release mode
    #[allow(unused_mut)]
    let mut exe_path;

    #[cfg(debug_assertions)] {
        exe_path = std::env::current_dir().expect("Failed to get working dir");
    }

    #[cfg(not(debug_assertions))] {
        let exe_path_owned = std::env::current_exe().expect("Cant' get executable location");
        exe_path = exe_path_owned.parent().expect("Uuh?").to_owned();
    }

    let mut default_dir_path = exe_path.clone();
    default_dir_path.push("default");
    default_dir_path.push("default.cowx");

    log::log("Parsing cowx files...");
    custom_tags_hash = parse_cowx_file(default_dir_path.to_str().expect("Uuh?"), custom_tags_hash, &args, true, &exe_path)?;

    // Cowx file from command line
    let cowx_file = matches.get_one::<String>("cowx");
    match cowx_file {
        Some(file_name) => {
            match parse_cowx_file(file_name, custom_tags_hash, &args, false, &exe_path) {
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

            let context = Context {
                args: &args,
                custom_tags: custom_tags_hash,
                ignore_aliases: false,
                default_dir: &exe_path,
            };

            let res = compile_file(path, content, context);

            match res {
                Ok(_) => {},
                Err(_) => {
                    log::log("No files produced.");
                },
            }
        },
        Err(err) => {
            log::error(&format!("failed to read source file: {}", err));
        },
    }

    return Ok(());
}


fn compile_file(absolute_path: PathBuf, content: String, mut context: Context) -> Result<(), ()> {
    log::log("Parsing document...");
    let document = match parser::parse_file(&absolute_path, &content.chars().collect(), &context) {
        Ok(node) => node,
        Err(err) => {
            log::error_position(&err.message, &err.position, err.length);
            return Err(());
        },
    };
    
    log::log("Creating HTML...");
    let (text, options) = match writer::get_file_text(document, &mut context) {
        Ok(res) => res,
        Err(_) => {
            return Err(());
        },
    };

    // Remove filename form path and add
    let mut out_path = absolute_path.parent().unwrap().to_path_buf();
    out_path.push("out.html");
    fs::write(out_path.clone(), text).unwrap();

    // Render to pdf!
    if context.args.no_pdf {
        log::log("No PDF created because you used --no-pdf");
    }
    else {
        let res = browser::render_to_pdf(out_path, context.args, &options);
        match res {
            Ok(()) => {},
            Err(()) => {
                log::log("Failed to create PDF file, but the HTML file have been created.")
            },
        }
    }

    log::log("Done!");
    return Ok(());
}


pub fn parse_cowx_file(file_name: &str, custom_tags_hash: HashMap<String, CustomTag>, arguments: &Args, is_default: bool, exe_path: &PathBuf) -> Result<HashMap<String, CustomTag>, ()> {
    match std::fs::read_to_string(file_name) { // Try to read the file
        Ok(content) => {
            // Parse the file
            let res_hash = parser::custom::parse_custom_tags(
                &content.chars().collect::<Vec<char>>(), 
                &mut parser::get_start_of_file_position(PathBuf::from(file_name)), 
                custom_tags_hash, 
                &arguments,
                is_default,
                exe_path
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

