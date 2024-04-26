
use std::path::PathBuf;

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
    pub js_files: Vec<DocumentPath>,
    pub cowx_files: Vec<DocumentPath>,
    pub footer_file: Option<DocumentPath>,
    pub header_file: Option<DocumentPath>,
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
        js_files: Vec::new(), // No additional css files linked by default
        cowx_files: Vec::new(),
        footer_file: None,
        header_file: None,
    };
    
    for child in &head.children {
        let inner_text = crate::parser::get_node_content_as_str(child);

        match child.name.as_str() {
            "title" => {
                res.title = inner_text;
            },
            "format" => {
                let format = get_format_from_name(inner_text);

                match crate::parser::get_attribute_value(child, "orientation") {
                    Ok(Some("portrait")) | Err(_) => {
                        res.format = format;
                    },
                    Ok(Some("landscape")) => {
                        res.format = DocFormat { width: format.height, height: format.width };
                    },
                    Ok(Some(str)) => {
                        log::warning(&format!("Invalid value for orientation attribute: \"{}\"", str));
                    },
                    Ok(None) => {
                        log::warning("Orientation attribute must have a value.");
                    },
                }
            },
            "paper-width" => {
                match inner_text.parse::<f32>() {
                    Ok(val) => {
                        if val <= 0.0 {
                            log::warning_position("Expected a positive value.", &child.start_position, child.source_length);
                        }
                        else {
                            res.format.width = val;
                        }
                    },
                    Err(_) => {
                        log::warning_position("The tag inner content should be a number.", &child.start_position, child.source_length);
                    },
                }
            },
            "paper-height" => {
                match inner_text.parse::<f32>() {
                    Ok(val) => {
                        if val <= 0.0 {
                            log::warning_position("Expected a positive value.", &child.start_position, child.source_length);
                        }
                        else {
                            res.format.height = val;
                        }
                    },
                    Err(_) => {
                        log::warning_position("The tag inner content should be a number.", &child.start_position, child.source_length);
                    },
                }
            }
            "css" => {
                res.css_files.push(get_doc_path_from_tag(child, inner_text));
            },
            "js" => {
                res.js_files.push(get_doc_path_from_tag(child, inner_text));
            },
            "cowx" => {
                res.cowx_files.push(get_doc_path_from_tag(child, inner_text));
            },
            "footer" => {
                res.footer_file = Some(get_doc_path_from_tag(child, inner_text));
            },
            "header" => {
                res.header_file = Some(get_doc_path_from_tag(child, inner_text));
            },
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
    let default = DocFormat { width: 210.0, height: 297.0 };  // Default is A4

    let format = text.to_lowercase();

    if format.len() == 2 {
        let (dim_a, dim_b): (f32, f32) = match format.chars().nth(0) {
            Some('a') => (26.0, 37.0),
            Some('b') => (31.0, 44.0),
            Some('c') => (28.0, 40.0),
            _ => {
                log::warning(&format!("Unknown paper format \"{}\". Using A4 by default.", format));
                return default;
            },
        };

        let nb = match format[1..].parse::<u32>() {
            Ok(n) => n,
            Err(_) => {
                log::warning(&format!("Unknown paper format \"{}\". Using A4 by default.", format));
                return default;
            },
        };

        if nb > 10 {
            log::warning(&format!("You tried to use the format \"{}\". This is ridiculous.", format));
            return default;
        }

        if nb % 2 == 0 {
            return DocFormat {
                width: dim_a * 2i32.pow((10 - nb) / 2) as f32,
                height: dim_b * 2i32.pow((10 - nb) / 2) as f32,
            };
        }
        else {
            return DocFormat {
                width: dim_b * 2i32.pow((9 - nb) / 2) as f32,
                height: dim_a * 2i32.pow((11 - nb) / 2) as f32,
            };
        }
    }
    else {
        log::warning(&format!("Unknown paper format \"{}\". Using A4 by default.", format));
        return default;
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
    /// Get the full path. Can throw an error via the log module, in this case it can return the wrong path.
    pub fn get_full_path(&self, context: &Context) -> PathBuf {
        match self.path_type {
            PathType::RelativeToFile => {
                let mut res = context.main_file_path.parent().unwrap().to_path_buf();
                res.push(PathBuf::from(self.path.clone()));
                return res;
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

