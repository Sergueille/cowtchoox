
use std::collections::HashMap;
use crate::parser;
use crate::parser::{Node, FilePosition, TagSymbol};

use super::{ParseError, TagAttribute};

/// Represents a tag created by the user. Also used for math operators
#[derive(Clone)]
pub struct CustomTag {
    pub arguments: Vec<String>,
    pub is_math: bool,
    pub content: Node,
    pub alias: Option<String>,
    pub infix_alias: bool
}


/// Stores all custom tags, in a big hash, the key is the name of the tag
pub type TagHash = HashMap<String, CustomTag>;


/// Put all defined tags in the text into the provided hashMap
/// 
/// # Arguments
/// * `file`: th content of the file
/// * `pos`: the positon in the file
/// * `hash`: the hash into which the function will add tags
/// * `is_default`: is it a default file (will ignore aliases etc.)
/// * `file_path`: the path to the file name 
///
pub fn parse_custom_tags(file: &Vec::<char>, pos: &mut FilePosition, hash: TagHash<>, args: &crate::Args, is_default: bool, 
    default_dir: &std::path::PathBuf, file_path: &std::path::PathBuf) -> Result<TagHash, parser::ParseError> {
    let mut context = parser::Context { args, custom_tags: hash, ignore_aliases: is_default, default_dir, main_file_path: file_path };

    loop { // Repeat until end of the file
        let mut node = parser::parse_tag(file, pos, TagSymbol::QUESTION_MARK | TagSymbol::EXCLAMATION_MARK, false, &context)?;

        // Check if a "?" was added
        let is_math = node.declaration_symbol == TagSymbol::QUESTION_MARK; 

        // Parse math immediately
        super::math::parse_all_math(&mut node, is_math, &context)?;

        let mut alias = None;
        let mut infix_alias = false;

        let mut arguments = Vec::with_capacity(node.attributes.len());
        for attr in &node.attributes {

            let mut chars = attr.name.chars();
            if chars.next() == Some(':') { // It's an argument
                if attr.value.is_some() {
                    return Err(parser::ParseError {
                        message: format!(
                            "In custom tag definition, the argument \"{}\" has value \"{}\", but it shouldn't have any. You should remove either the colon to make it a regular attribute, or the value.", 
                            attr.name, 
                            attr.value.clone().unwrap()
                        ),
                        position: node.start_position.clone(),
                        length: node.start_inner_position.absolute_position - node.start_position.absolute_position
                    });
                }

                let arg_name = chars.collect();
                arguments.push(arg_name);
            }
            else if attr.name == "alias" { // "alias" attribute

                // Prevent two alias attributes
                if alias != None {
                    return Err(ParseError {
                        message: String::from("Custom tag can have only 1 alias. Two alias attributes were found."),
                        position: attr.position.clone().expect("Error probably because the attribute is created by internal code..."),
                        length: attr.name.chars().count(),
                    });
                }

                match &attr.value {
                    Some(value) => alias = Some(value.clone()),
                    // Throw error if no value
                    None => return Err(ParseError {
                        message: String::from("The attribute alias is used to define an alias for the tag, so the attribute should have a value."),
                        position: attr.position.clone().expect("Error probably because the attribute is created by internal code..."),
                        length: attr.name.chars().count(),
                    }),
                }
            }
            else if attr.name == "infix-alias" {
                infix_alias = true;
            }
            else {
                // Real attribute: do nothing
            }
        }
        
        // Check for incorrect or missing colon tags inside
        check_colon_tags(&node, &arguments)?;

        context.custom_tags.insert(node.name.clone(), CustomTag {
            arguments,
            is_math,
            content: node,
            alias,
            infix_alias,
        }); 

        match super::advance_until_non_whitespace(file, pos) {
            Ok(()) => {},
            Err(_) => break,
        }
    }

    return Ok(context.custom_tags);
}


/// Returns the cloned contents of the tag, with args tags replaced by their values 
/// 
/// # Arguments
/// * `tag`: the tag to instantiate
/// * `arguments`: a list of argument values, provided in the right order
/// 
pub fn instantiate_tag(tag: &CustomTag, arguments: Vec<Node>) -> Node {
    return instantiate_tag_inner(tag, &tag.content, &arguments, true);
}


/// Returns the cloned contents of the tag, with args tags replaced by their values 
/// Same as `instantiate_tag_with_named_parameters`, but with named parameters
/// 
/// # Arguments
/// * `tag`: the tag to instantiate
/// * `arguments`: a list of argument names and values
/// 
pub fn instantiate_tag_with_named_parameters(tag: &CustomTag, arguments: Vec<(String, Node)>, pos: &FilePosition) -> Result<Node, ParseError> {
    let mut arg_values = vec![None; tag.arguments.len()];

    for (name, value) in arguments {
        // Search for the argument position
        for (i, arg) in tag.arguments.iter().enumerate() {
            if arg == &name {
                if arg_values[i].is_some() {
                    return Err(ParseError {
                        message: format!("Argument \"{}\" provided twice.", name),
                        position: value.start_position.clone(),
                        length: 1,
                    })
                }

                arg_values[i] = Some(value);
                break;
            }
        }
    }

    // Collect arguments, and check if they are all there 
    let mut final_arguments = Vec::with_capacity(arg_values.len());
    for (i, arg) in arg_values.into_iter().enumerate() {
        match arg {
            Some(a) => final_arguments.push(a),
            None => return Err(ParseError {
                message: format!("Tag argument \"{}\" was not provided.", tag.arguments[i]),
                position: pos.clone(),
                length: 1,
            }),
        }
    }

    let res = instantiate_tag_inner(tag, &tag.content, &final_arguments, true);
    return Ok(res);
}


fn instantiate_tag_inner(tag: &CustomTag, node: &Node, arguments: &Vec<Node>, copy_arguments_into_attributes: bool) -> Node {
    // Copy argument values in attributes
    let mut res_attibutes = Vec::new();
    for i in 0..node.attributes.len() {
        let attr = &node.attributes[i];

        if attr.name.chars().next() == Some(':') && copy_arguments_into_attributes {
            res_attibutes.push(TagAttribute {
                name: attr.name.clone(),
                value: Some(crate::parser::get_node_content_as_str(&arguments[i])),
                position: attr.position.clone(),
                value_position: attr.value_position.clone(),
            });
        }
        else {
            res_attibutes.push(attr.clone());
        }
    }

    let mut res = Node {
        name: node.name.clone(),
        attributes: res_attibutes,
        children: node.children.clone(),
        content: Vec::with_capacity(node.content.len()),
        auto_closing: node.auto_closing,
        is_math: false,
        declaration_symbol: TagSymbol::NOTHING, 
        start_position: node.start_position.clone(),
        start_inner_position: node.start_inner_position.clone(),
        source_length: node.source_length,
    };

    for c in &node.content {
        match c {
            super::NodeContent::Character(c) => {
                res.content.push(super::NodeContent::Character(c.clone()));
            },
            super::NodeContent::EscapedCharacter(c) => {
                res.content.push(super::NodeContent::EscapedCharacter(c.clone()));
            },
            super::NodeContent::Child(child_id) => {
                let child = &node.children[*child_id];

                let mut replaced_argument = false;
                if child.auto_closing && child.declaration_symbol == TagSymbol::COLON {
                    // Should this tag be replaced by an argument?
                    for (i, arg) in tag.arguments.iter().enumerate() {
                        if arg == &child.name {
                            replaced_argument = true;
                            res.children[*child_id] = arguments[i].clone();
                        }
                    }
                }

                res.content.push(super::NodeContent::Child(*child_id));

                if !replaced_argument {
                    let new_child = instantiate_tag_inner(tag, child, arguments, false);
                    res.children[*child_id] = new_child;
                }
            },
        }
    }

    return res;
}


/// Does this custom tag have an :inner argument?  
pub fn has_inner_param(tag: &CustomTag) -> bool {
    for arg in &tag.arguments {
        let mut chars = arg.chars();
        chars.next(); // Skip the colon at the beginning
        if arg == "inner" {
            return true;
        }
    }

    return false;
}

// Returns error if finds colon tags which name is NOT in the list 
pub fn check_colon_tags(node: &Node, allowed_arguments: &Vec<String>) -> Result<(), ParseError> {
    for child in &node.children {
        if child.declaration_symbol == super::TagSymbol::COLON {
            if allowed_arguments.contains(&child.name) {
                // Ok, the argument exists!
            }
            else {
                return Err(ParseError {
                    message: format!("\
Unknown parameter \"{}\" used. You may have forgotten to add it in the custom tag declaration. \
If you meant to use a regular tag, remove the colon.", child.name),
                    position: child.start_position.clone(),
                    length: child.name.chars().count() + 2,
                });
            }
        }
        else if !child.auto_closing {
            check_colon_tags(child, allowed_arguments)?;
        }
    }

    return Ok(());
}



