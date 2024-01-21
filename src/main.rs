
mod parser;
mod writer;
mod doc_options;

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
            compile_file(filepath, content);
        },
        Err(err) => {
            println!("ERR: failed to read source file: {}", err);
        },
    }

    println!("Finished!");
}

fn compile_file(file_path: &String, content: String) {
    // TEST: testing parser here

    let document = parser::parse_file(file_path, &content.chars().collect()).expect("Failed to parse file");
    let text = writer::get_file_text(&document).expect("Failed to create HTML");

    println!("Parsed struct: \n{:?}", document);
    println!("HTML output: \n{}", text);
}

