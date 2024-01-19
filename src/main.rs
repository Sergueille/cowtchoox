// Interpret command line arguments

use clap;

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

    println!("The file is: {}", filepath)
    
    // TODO: do the rest fo the entire program
}

