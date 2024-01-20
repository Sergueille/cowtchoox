
mod parser;

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
            compile_file(content);
        },
        Err(err) => {
            println!("ERR: failed to read source file: {}", err);
        },
    }

    println!("Finished!");
}

fn compile_file(content: String) {
    // TEST: testing parser here

    let res = parser::parse_file(&content.chars().collect());
    match res {
        Ok(node) => println!("Parsed file: {:?}", node),
        Err(err) => println!("Failed to parse file: {:?}", err),
    }

    // TODO: do the actual work
}

