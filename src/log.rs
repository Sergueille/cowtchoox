use std::fs;

use crate::util::FilePosition;
use colored::Colorize;


// This file will show errors in the terminal


pub enum LogLevel {
    Log, Warning, Error
}


/// Prints a log
pub fn log(message: &str) {
    println!("    {} {}", "log:".blue(), message);
}


/// Prints a log with a position
/// NOTE: the highlighted position must be on one line
pub fn log_position(message: &str, start_pos: &FilePosition, length: usize) {
    println!("    {} {} {}", "log:".blue(), message, display_path(start_pos).bright_black());
    print_line(start_pos, length, LogLevel::Log);
}


/// Prints a warning
pub fn warning(message: &str) {
    println!("{} {}", "warning:".yellow(), message);
}


/// Prints a warning
/// NOTE: the highlighted position must be on one line
pub fn warning_position(message: &str, start_pos: &FilePosition, length: usize) {
    println!("{} {} {}", "warning:".yellow(), message, display_path(start_pos).bright_black());
    print_line(start_pos, length, LogLevel::Warning);
}


/// Prints an error
pub fn error(message: &str) {
    println!("  {} {}", "error:".red(), message);
}


/// Prints an error
/// NOTE: the highlighted position must be on one line
pub fn error_position(message: &str, start_pos: &FilePosition, length: usize) {
    println!("  {} {} {}", "error:".red(), message, display_path(start_pos).bright_black());
    print_line(start_pos, length, LogLevel::Error);
}


/// Displays the message if an error is received
pub fn log_if_err<T, E>(res: Result<T, E>, message: &str) -> Result<T, ()> {
    return res.or_else(|_| { error(message); Err(()) });
}


fn display_path(start_pos: &FilePosition) -> String {
    return format!("at {}:{}:{}", start_pos.file_path.to_str().unwrap(), start_pos.line + 1, start_pos.line_character);
}


fn print_line(start_pos: &FilePosition, length: usize, level: LogLevel) {
    let content = fs::read_to_string(start_pos.file_path.clone()).unwrap();

    let line = content.lines().nth(start_pos.line).expect("The line doesn't exists in the file!");
    let line_number_text = (start_pos.line + 1).to_string();

    let spaces_vector = vec![b' '; line_number_text.len()];
    let spaces = String::from_utf8_lossy(&spaces_vector); // Create a string of spaces with the same size as the line number

    // Create underline text
    let mut underline = String::with_capacity(line.len());
    for _ in 0..start_pos.line_character {
        underline.push(' ');
    }
    for _ in 0..length {
        underline.push('~');
    }

    let colored_underline = match level {
        LogLevel::Log => underline.blue(),
        LogLevel::Warning => underline.yellow(),
        LogLevel::Error => underline.red(),
    };

    println!("         {} {}", spaces, "|".bright_black());
    println!("         {} {} {}", line_number_text.bright_black(), "|".bright_black(), line);
    println!("         {} {} {}", spaces, "|".bright_black(), colored_underline);
}
