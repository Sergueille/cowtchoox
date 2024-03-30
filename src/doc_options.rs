
use std::{fs, path::PathBuf};

use crate::{log, parser::Node, Context};


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
    pub css_files: Vec<DocumentPath>,
    pub cowx_files: Vec<DocumentPath>,
    pub footer_file: Option<DocumentPath>
}


/// Represents a path specified in a .cow file
pub struct DocumentPath {
    pub path: String,
    pub path_type: PathType,
}


pub enum PathType {
    RelativeToFile,
    Absolute,
    RelativeToDefaultDir
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
        cowx_files: Vec::new(),
        footer_file: None,
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
                res.css_files.push(get_doc_path_from_tag(child, inner_text));
            },
            "cowx" => {
                res.cowx_files.push(get_doc_path_from_tag(child, inner_text));
            },
            "footer" => {
                res.footer_file = Some(get_doc_path_from_tag(child, inner_text));
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


fn get_doc_path_from_tag(tag: &Node, inner_content: String) -> DocumentPath {
    let mut path_type = PathType::RelativeToFile; // Default value

    for attr in & tag.attributes {
        if attr.name == "relative-to" {
            match &attr.value {
                Some(val) => {
                    if val == "absolute" {
                        path_type = PathType::Absolute;
                    }
                    else if val == "file" {
                        path_type = PathType::RelativeToFile;
                    }
                    else if val == "default-dir" {
                        path_type = PathType::RelativeToDefaultDir;
                    }
                    else {
                        log::warning_position(&format!(
                            "Unknown value \"{}\" of \"relative-to\" attribute. Use either \"absolute\", \"file\", or \"default-dir\"", val), 
                            attr.position.as_ref().unwrap(), attr.name.len()
                        );
                    }
                },
                None => {
                    log::warning_position("This attribute should have a value. You can remove it de keep default.", attr.position.as_ref().unwrap(), attr.name.len());
                },
            }
        }
    }

    return DocumentPath { path: inner_content, path_type }
}   


impl DocumentPath {
    pub fn get_full_path(&self, context: &Context) -> PathBuf {
        match self.path_type {
            PathType::RelativeToFile => {
                return fs::canonicalize(PathBuf::from(self.path.clone())).unwrap(); // TODO: report error correctly
            },
            PathType::Absolute => {
                return PathBuf::from(self.path.clone());
            },
            PathType::RelativeToDefaultDir => {
                let mut path = context.default_dir.clone();
                path.push(self.path.clone());
                return path;
            },
        }
    }
}

