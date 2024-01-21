
use crate::parser::Node;


// Handle document options


pub enum DocFormat {
    A4, // TODO: add weird formats
}


/// Everything you want to know about the document
pub struct DocOptions {
    pub title: String,
    pub format: DocFormat,
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
        format: DocFormat::A4,
    };
    
    for child in &head.children {
        let inner_text = crate::parser::get_node_content_as_str(child);

        match child.name.as_str() {
            "title" => {
                res.title = inner_text;
            },
            "format " => {
                res.format = get_doc_format(inner_text);
            },
            _ => {
                // TODO: warn for unknown tag
            }
        }
    };

    return res;
}


fn get_doc_format(text: String) -> DocFormat {
    match text.to_lowercase().as_str() {
        "a4" => {
            return DocFormat::A4;
        }
        _ => {
            // TODO: warn
            return DocFormat::A4; // Default
        }
    }
}

