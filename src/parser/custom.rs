
use std::collections::HashMap;
use crate::parser;
use crate::parser::{Node, FilePosition, TagSymbol};

use super::ParseError;

/// Represents a tag created by the user. Also used for math operators
pub struct CustomTag {
    pub arguments: Vec<String>,
    pub is_math: bool,
    pub content: Node,
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
///
pub fn parse_custom_tags(file: &Vec::<char>, pos: &mut FilePosition, hash: TagHash<>, args: &crate::Args, is_default: bool) -> Result<TagHash, parser::ParseError> {
    let mut context = parser::Context { args, custom_tags: hash, ignore_aliases: is_default };

    loop { // Repeat until end of the file
        let node = parser::parse_tag(file, pos, TagSymbol::QUESTION_MARK | TagSymbol::EXCLAMATION_MARK, false, true, &context)?;

        // Check if a "?" was added
        let is_math = node.declaration_symbol == TagSymbol::QUESTION_MARK; 

        let mut arguments = Vec::with_capacity(node.attributes.len());
        for (name, value) in &node.attributes {

            let mut chars = name.chars();
            if chars.next() == Some(':') { // It's an argument
                if value != "" {
                    return Err(parser::ParseError {
                        message: format!("In custom tag definition, the argument \"{}\" has value \"{}\". You should remove either the colon or the value.", name, value),
                        position: node.start_position.clone(),
                        length: node.start_inner_position.absolute_position - node.start_position.absolute_position
                    });
                }

                let arg_name = chars.collect();
                arguments.push(arg_name);
            }
            else {
                // Real attribute: do nothing
            }
        }
        
        context.custom_tags.insert(node.name.clone(), CustomTag {
            arguments,
            is_math,
            content: node,
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
    return instantiate_tag_inner(tag, &tag.content, &arguments);
}


/// Returns the cloned contents of the tag, with args tags replaced by their values 
/// Same as `instantiate_tag_with_named_parameters`, but with named parameters
/// 
/// # Arguments
/// * `tag`: the tag to instantiate
/// * `arguments`: a list of argument names and values
/// 
pub fn instantiate_tag_with_named_parameters(tag: &CustomTag, arguments: Vec<(String, Node)>, pos: &FilePosition) -> Result<Node, ParseError> {
    let mut arg_values = vec![None; arguments.len()];

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

    return Ok(instantiate_tag_inner(tag, &tag.content, &final_arguments));
}


fn instantiate_tag_inner(tag: &CustomTag, node: &Node, arguments: &Vec<Node>) -> Node {
    let mut res = Node {
        name: node.name.clone(),
        attributes: node.attributes.clone(),
        children: node.children.clone(),
        content: Vec::with_capacity(node.content.len()),
        auto_closing: node.auto_closing,
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
                if child.auto_closing {
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
                    let new_child = instantiate_tag_inner(tag, child, arguments);
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




