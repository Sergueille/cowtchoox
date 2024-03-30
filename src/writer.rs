
use std::fs;
use std::path::PathBuf;

use crate::log;
use crate::Context;
use crate::parser::{Node, NodeContent, ParseError};
use crate::parser::custom;
use crate::doc_options::{self, DocOptions};

// Transform the struct back to raw HTML
// NOTE: all text will be wrapped in <text> tags


// Get the entire text of the document, ready for being displayed
pub fn get_file_text(mut document: Node, context: &mut Context) -> Result<(String, DocOptions), ()> {
    let mut res = String::new();

    let head = match try_get_children_with_name(&mut document, "head") {
        Ok(head) => head,
        Err(()) => {
            log::error("The document has no head.");
            return Err(());
        }
    };
    let options = doc_options::get_options_form_head(head);

    // Look for additional cowx files listed in head
    for cowx_file in &options.cowx_files {
        let content = match fs::read_to_string(cowx_file.get_full_path(context)) {
            Ok(content) => content,
            Err(err) => {
                log::error(
                    &format!(
                        "Could not read cowx file \"{}\" specified in document head. ({}) Make sure the path is relative to the compiled file.", 
                        cowx_file.get_full_path(context).display(), err
                    )
                );
                return Err(());
            },
        };

        // Parse the file!
        match custom::parse_custom_tags(
            &content.chars().collect(), 
            &mut crate::parser::get_start_of_file_position(PathBuf::from(cowx_file.get_full_path(context))), 
            std::mem::replace(&mut context.custom_tags, std::collections::HashMap::new()),
            &context.args, 
            false,
            context.default_dir
        ) {
            Ok(res) => context.custom_tags = res,
            Err(err) => {
                log::error_position(&err.message, &err.position, err.length);
            },
        }
    } 

    let mut finished_document = parse_math_and_replace_tags(document, &context)?;

    // Get the body from the document
    let body = match try_get_children_with_name(&mut finished_document, "body") {
        Ok(res) => res,
        Err(()) => {
            log::error("The document has no body");
            return Err(());
        }
    };

    // Parse the header, if found add it as a child to the body
    match &options.footer_file {
        Some(file) => {
            let file_res = std::fs::read_to_string(file.get_full_path(context));
            match file_res {
                Ok(string) => {
                    let parsed = crate::parser::parse_file(&PathBuf::from(file.get_full_path(context).clone()), &string.chars().collect(), context);
                    match parsed {
                        Ok(mut node) => {
                            // Add the footer as a child
                            node.name = String::from("doc-footer");
                            let finished_footer = parse_math_and_replace_tags(node, context)?;

                            body.content.push(crate::parser::NodeContent::Child(body.children.len()));
                            body.children.push(finished_footer);
                        },
                        Err(err) => {
                            log::error_position(&err.message, &err.position, err.length);
                            return Err(());
                        },
                    }
                },
                Err(err) => {
                    log::error(&format!("Failed to read the header file: {}", err));
                    return  Err(());
                },
            }
        },
        None => {},
    };    

    res.push_str("<html>"); // Quirks is better!

    res.push_str(&white_head(&options, &context));

    // Write the body text
    res.push_str(&get_node_html(&body, false, &context));

    res.push_str("</html>");

    return Ok((res, options));
}


fn parse_math_and_replace_tags(node: Node, context: &Context) -> Result<Node, ()> {
    // Instantiate the custom tags used in the document
    let mut with_custom_tags = match instantiate_all_custom_tags(node, false, context) {
        Ok(node) => node,
        Err(err) => {
            log::error_position(&err.message, &err.position, err.length);
            return Err(());
        },
    };

    // Parse the math
    match crate::parser::math::parse_all_math(&mut with_custom_tags, false, context) {
        Ok(()) => {},
        Err(err) => {
            log::error_position(&err.message, &err.position, err.length);
            return Err(());
        },
    };

    return Ok(with_custom_tags);
}


pub fn white_head(options: &doc_options::DocOptions, context: &Context) -> String {
    let mut res = String::with_capacity(200);
    res.push_str("<head>");

    // Document title
    res.push_str(format!("<title>{}</title>", options.title).as_str());

    // FIXME: should be like ~"path_to_exe/" when built, and ~"" when running with cargo
    //        but too lazy to do that
    let default_resources_path = context.default_dir.to_str().expect("Failed to get resources dir string").to_string().replace("\\", "/");

    // Link JS script, so that it executes when the page loads
    res.push_str(&format!("<script defer=\"defer\" src=\"file:///{}/JS/main.js\"></script>", default_resources_path));

    // Link default CSS
    res.push_str(&format!("<link rel=\"stylesheet\" href=\"file:///{}/default/util.css\"/>", default_resources_path));
    res.push_str(&format!("<link rel=\"stylesheet\" href=\"file:///{}/default/default.css\"/>", default_resources_path));

    // Link additional CSS
    for file_path in &options.css_files {
        let path_str = crate::util::get_browser_path_string(file_path.get_full_path(context));
        res.push_str(&format!("<link rel=\"stylesheet\" href=\"{}\"/>", path_str));
    }

    // IMPORTANT NOTE: make sure this tag is the last CSS tag, to make sure users don't accidentally change critical CSS rules (such as pag elements) 
    res.push_str(&format!("<link rel=\"stylesheet\" href=\"file:///{}/default/critical.css\"/>", default_resources_path));

    // Page size
    res.push_str(&format!("<meta name=\"pagewidth\" content=\"{}\"/>", options.format.width));
    res.push_str(&format!("<meta name=\"pageheight\" content=\"{}\"/>", options.format.height));

    res.push_str("</head>");
    return res;
}


/// Looks for the head of a document, returns Err if not found
fn try_get_children_with_name<'a>(document: &'a mut Node, name: &str) -> Result<&'a mut Node, ()> {
    for child in &mut document.children {
        if child.name == name {
            return Ok(child);
        }
    }

    return Err(());
}


/// Generates HTML for a node
///
/// # Arguments
/// * `no_text_tags`: will not create <text> tags (for pre of svg)
pub fn get_node_html(node: &Node, no_text_tags: bool, context: &Context) -> String {
    let mut res = String::from("<");

    res.push_str(&node.name);

    res.push(' ');

    for attr in &node.attributes {
        match &attr.value {
            Some(val) => {
                res.push_str(&format!("{}=\"{}\" ", &attr.name, val));
            },
            None => {
                res.push_str(&format!("{} ", &attr.name));
            },
        };
    }    

    if node.auto_closing {
        res.push_str("/>");
    }
    else {
        res.push('>');

        let mut inner_html = String::new();

        let mut in_text = false;
        let mut current_text_tag = String::new(); // Accumulate text here, and push it at the end, or when a child is encountered

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

                    inner_html.push_str(&get_node_html(&node.children[*id], no_text_tags || node.children[*id].name == "svg" || node.children[*id].name == "pre", context))
                },
            }
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

        if node.name == "pre" {
            inner_html = inner_html.trim().to_string();
        }

        res.push_str(&format!("{}</{}>", inner_html, &node.name))
    }

    return res;
}


/// Looks for custom tags in document, then replaces them with their definition
pub fn instantiate_all_custom_tags(mut node: Node, only_children: bool, context: &Context) -> Result<Node, ParseError> {
    // Put children in an option array
    let owned_children = std::mem::replace(&mut node.children, Vec::new());
    let mut opt_children : Vec<_> = owned_children.into_iter().map(|c| Some(c)).collect();

    for content in &node.content {
        match content {
            NodeContent::Child(id) => {
                let child = std::mem::replace(&mut opt_children[*id], None).unwrap();
                let changed = instantiate_all_custom_tags(child, false, context)?; // Instantiate tags inside children
                opt_children[*id] = Some(changed);
            },
            _ => {},
        }
    }

    // Put the children back
    node.children = opt_children.into_iter().map(|opt| opt.unwrap()).collect();
    
    // Now, if it's a custom tag, instantiate it properly
    if !only_children && node.declaration_symbol == crate::parser::TagSymbol::EXCLAMATION_MARK  {
        let custom_tag = match context.custom_tags.get(&node.name) {
            Some(tag) => tag,
            None => {
                return Err(ParseError {
                    message: format!("Unknown custom tag \"{}\" used.", node.name),
                    position: node.start_position,
                    length: node.name.len() + 1,
                });
            }
        };

        if custom_tag.is_math {
            return Err(ParseError {
                message: format!("You tried to use \"{}\" as a custom tag, but it has been declared as a math operator. Use it with the math operator syntax.", node.name),
                position: node.start_position,
                length: node.name.len() + 1,
            });  
        }

        let mut arguments = Vec::with_capacity(node.attributes.len() + 1);
        for attr in node.attributes.iter() {
            let mut chars = attr.name.chars();
            if chars.next().unwrap() == ':' {
                match &attr.value {
                    Some(val) => {
                        let mut val_pos = attr.value_position.clone().expect("The tag argument does not come from source file!");

                        // HACK: put a space after to prevent the parser from complaining it gets the end of the string
                        let mut padded_val = val.clone();
                        padded_val.push(' ');
                        let node = crate::parser::get_tag_from_raw_text(&padded_val, custom_tag.is_math, &mut val_pos, context)?;
                        arguments.push((chars.collect(), node));
                    },
                    None => {
                        return Err(ParseError {
                            message: format!("The argument {} has no value. You should add a value after: \"{}='value'\". If you meant to add a regular attribute, you should remove the colon.", attr.name, attr.name),
                            position: attr.position.clone().expect("The tag argument does not come from source file!"),
                            length: attr.name.len(),
                        });
                    }
                }
            } 
        }

        let start_position = node.start_position.clone();

        let has_inner = custom::has_inner_param(custom_tag);
        if node.auto_closing {
            if has_inner {
                return Err(ParseError {
                    message: format!("The custom tag \"{}\" should not be auto-closing. You should usee it like this: \"<!{}></{}>\".", node.name, node.name, node.name),
                    position: node.start_position,
                    length: node.name.len() + 1,
                });  
            }
        }
        else {
            if !has_inner {
                return Err(ParseError {
                    message: format!("The custom tag \"{}\" should be auto-closing. You should usee it like this: \"<!{}/>\".", node.name, node.name),
                    position: node.start_position.clone(),
                    length: node.name.len() + 1,
                });  
            }

            // Remove attributes and change name
            node.name = String::from("inner");
            node.attributes = Vec::new();

            arguments.push((String::from("inner"), node)); // Push the inner content as an ":inner" argument
        }

        let actual_res = custom::instantiate_tag_with_named_parameters(custom_tag, arguments, &start_position)?;

        return Ok(actual_res);
    }
    else {
        return Ok(node);
    }
}
