
use std::collections::HashMap;
use crate::parser;
use crate::parser::{Node, FilePosition};

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
/// 
/// TODO: this function is not tested at all
pub fn parse_custom_tags(file: &Vec::<char>, pos: &mut FilePosition, hash: TagHash<>, args: &crate::Args) -> Result<TagHash, parser::ParseError> {
    // TODO: not finished

    let mut context = parser::ParserContext { args, math_operators: hash };

    while pos.absolute_position < file.len() { // Repeat until end of the file
        let node = parser::parse_tag(file, pos, true, false, &context)?;

        // Check if a "?" was added
        let is_math = parser::get_attribute_value(&node, parser::MATH_OPERATOR_ATTRIB_NAME).is_ok(); 

        let mut arguments = Vec::with_capacity(node.attributes.len());
        for (name, value) in &node.attributes {
            if name == parser::MATH_OPERATOR_ATTRIB_NAME {
                continue; // An internal thing. Just ignore
            }

            if value != "" {
                // TODO: report error: values ar not allowed
            }

            arguments.push(name.clone());
        }
        
        context.math_operators.insert(node.name.clone(), CustomTag {
            arguments,
            is_math,
            content: node,
        }); 

        super::advance_until_non_whitespace(file, pos);
    }

    return Ok(context.math_operators);
}


/// Returns the cloned contents of the tag, with args tags replaced by their values 
/// 
/// # Arguments
/// * `arguments`: a list of argument values, provided in the right order
/// 
pub fn instantiate_tag(tag: &CustomTag, arguments: Vec<Node>) {
    // TODO
}




