
use std::path::PathBuf;

use crate::log;
use crate::parser::{Node, NodeContent};
use crate::doc_options::{self, DocOptions};

// Transform the struct back to raw HTML
// NOTE: all text will be wrapped in <text> tags


// Get the entire text of the document, ready for being displayed
pub fn get_file_text(document: &Node, exe_path: PathBuf) -> Result<(String, DocOptions), ()> {
    let mut res = String::new();

    let head = match try_get_children_with_name(document, "head") {
        Ok(head) => head,
        Err(()) => {
            log::error("The document has no head.");
            return Err(());
        }
    };
    let options = doc_options::get_options_form_head(head);

    res.push_str("<html>"); // Quirks is better!

    res.push_str(&white_head(&options, exe_path));

    let body = match try_get_children_with_name(document, "body") {
        Ok(res) => res,
        Err(()) => {
            log::error("The document has no body");
            return Err(());
        }
    };

    res.push_str(&get_node_html(&body, false));

    res.push_str("</html>");

    return Ok((res, options));
}


pub fn white_head(options: &doc_options::DocOptions, exe_path: PathBuf) -> String {
    let mut res = String::with_capacity(200);
    res.push_str("<head>");

    // Document title
    res.push_str(format!("<title>{}</title>", options.title).as_str());

    // FIXME: should be like ~"path_to_exe/" when built, and ~"" when running with cargo
    //        but too lazy to do that
    let default_resources_path = exe_path.to_str().expect("Failed to get resources dir string").to_string().replace("\\", "/");

    // Link JS script, so that it executes when the page loads
    res.push_str(&format!("<script defer=\"defer\" src=\"file:///{}/JS/main.js\"></script>", default_resources_path));

    // Link additional CSS
    for file_path in &options.css_files {
        res.push_str(&format!("<link rel=\"stylesheet\" href=\"{}\"/>", file_path));
    }

    // Link default CSS
    // IMPORTANT NOTE: make sure this tag is the last CSS tag, to make sure users don't accidentally change critical CSS rules (such as pag elements) 
    res.push_str(&format!("<link rel=\"stylesheet\" href=\"file:///{}/default/util.css\"/>", default_resources_path));
    res.push_str(&format!("<link rel=\"stylesheet\" href=\"file:///{}/default/default.css\"/>", default_resources_path));
    res.push_str(&format!("<link rel=\"stylesheet\" href=\"file:///{}/default/critical.css\"/>", default_resources_path));

    // Page size
    res.push_str(&format!("<meta name=\"pagewidth\" content=\"{}\"/>", options.format.width));
    res.push_str(&format!("<meta name=\"pageheight\" content=\"{}\"/>", options.format.height));

    res.push_str("</head>");
    return res;
}


/// Looks for the head of a document, returns Err if not found
pub fn try_get_children_with_name<'a>(document: &'a Node, name: &str) -> Result<&'a Node, ()> {
    for child in &document.children {
        if child.name == name {
            return Ok(child);
        }
    }

    return Err(());
}


/// Generates HTML for a node
/// TEST: the function is not actually implemented properly
///       this is directly reconstructing the tag without processing anything
///
/// # Arguments
/// * `no_text_tags`: will not create <text> tags (for pre of svg)
pub fn get_node_html(node: &Node, no_text_tags: bool) -> String {
    let mut res = String::from("<");

    res.push_str(&escape_tag_name(&node.name));

    res.push(' ');
    
    for (attr, val) in &node.attributes {
        res.push_str(&format!("{}=\"{}\" ", &attr, &val));
    }    

    if node.auto_closing {
        res.push_str("/>");
    }
    else {
        res.push('>');

        let mut inner_html = String::new();

        let mut in_text = false;
        let mut current_text_tag = String::new(); // Accumulate text here, and push it at the end, or when a child is encountered

        let mut previous: &NodeContent = &NodeContent::Child(0); // Keep track of the last character
        for content in &node.content {
            match content {
                crate::parser::NodeContent::Character((c, _)) | NodeContent::EscapedCharacter((c, _)) => {
                    if !in_text {
                        in_text = true;
                    }

                    // Escape characters
                    if *c == '<' {
                        current_text_tag.push_str("&lt");
                    }
                    else if *c == '>' {
                        current_text_tag.push_str("&gt");
                    }
                    else if *c == '&' {
                        current_text_tag.push_str("&amp");
                    }
                    else {
                        current_text_tag.push(*c);
                    }
                },
                crate::parser::NodeContent::Child(id) => {
                    if in_text && current_text_tag.trim().len() != 0 {
                        if !no_text_tags {
                            inner_html.push_str("<text>");
                        }

                        inner_html.push_str(&current_text_tag);

                        if !no_text_tags {
                            inner_html.push_str("</text>");
                        }

                        in_text = false;
                        current_text_tag = String::new();
                    }

                    inner_html.push_str(&get_node_html(&node.children[*id], no_text_tags || node.children[*id].name == "svg" || node.children[*id].name == "pre"))
                },
            }

            previous = content;
        }

        if in_text && current_text_tag.trim().len() != 0 {
            if !no_text_tags {
                inner_html.push_str("<text>");
            }

            inner_html.push_str(&current_text_tag);

            if !no_text_tags {
                inner_html.push_str("</text>");
            }
        }
        
        if !no_text_tags {
            match previous {
                NodeContent::Character(_) | NodeContent::EscapedCharacter(_) => {
                    inner_html.push_str("</text>"); // End text tag
                },
                NodeContent::Child(_) => {},
            }
        }

        if node.name == "pre" {
            inner_html = inner_html.trim().to_string();
        }

        res.push_str(&format!("{}</{}>", inner_html, &escape_tag_name(&node.name)))
    }

    return res;
}


fn escape_tag_name(name: &str) -> String {
    if name == "math" {
        return String::from("mathnode");
    }
    else {
        return String::from(name);
    }
}
