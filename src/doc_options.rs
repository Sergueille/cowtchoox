
use crate::{log, parser::Node};


// Handle document options


/// Size of a page. Values are in mm
pub struct DocFormat {
    pub width: f32,
    pub height: f32,
}


/// Everything you want to know about the document
pub struct DocOptions {
    pub title: String,
    pub format: DocFormat,
    pub css_files: Vec<String>,
}


/// Takes the raw head node form the document, and extract the options
/// 
/// # Panics
/// panics if you pass an other node than head
/// 
pub fn get_options_form_head(head: &Node) -> DocOptions {
    if head.name != "head" {
        panic!("You must pass the head!");
    }

    let mut res = DocOptions { // Put default values here
        title: String::from("You forgot to specify the title!"),
        format: DocFormat { width: 210.0, height: 297.0 }, // Default to A4
        css_files: Vec::new(), // No additional css files linked by default
    };
    
    for child in &head.children {
        let inner_text = crate::parser::get_node_content_as_str(child);

        match child.name.as_str() {
            "title" => {
                res.title = inner_text;
            },
            "format" => {
                res.format = get_format_from_name(inner_text);
            },
            "css" => {
                res.css_files.push(inner_text);
            }
            tag_name => {
                log::warning_position(
                    &format!("Unknown tag \"{}\" in head.", tag_name), 
                    &child.start_position, 
                    child.source_length
                );
            }
        }
    };

    return res;
}


/// Converts the name found in the COW files to the right dimensions. Warns and returns A4 if not recognized
fn get_format_from_name(text: String) -> DocFormat {
    match text.to_lowercase().as_str() {
        "a4" => {
            return DocFormat { width: 210.0, height: 297.0 };
        }
        other_format => {
            log::warning(&format!("Unknown paper format \"{}\". Using A4 by default.", other_format));
            return DocFormat { width: 210.0, height: 297.0 }; // Default
        }
    }
}

