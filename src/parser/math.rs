use super::{advance_position, custom::instantiate_tag, Node, NodeContent, ParseError, ParserContext};
use crate::util::FilePosition;
use super::custom::CustomTag;


/// Create math! Called after tags are parsed. Will replace the provided Node's contents by math.
/// 
/// # Arguments
/// * `node`: A node. It's children are fully math-parsed, but not it's inner text
/// 
/// # Returns
/// The node that contains all the math.
/// 
/// TODO: absolutely not finished
pub fn parse_math(node: &mut Node, chars: &Vec<char>, context: &ParserContext) -> Result<(), ParseError> {
    let mut res: Vec<NodeContent> = Vec::with_capacity(node.content.len());

    let mut i = 0;
    while i < node.content.len() {
        let c = &node.content[i];

        match *c {
            NodeContent::Character(c) => {
                if c == '?' {
                    i += 1;
                    let op = expect_operator(node, chars, &mut i, context)?;

                    let mut arguments = Vec::with_capacity(op.arguments.len());
                    for _ in 0..op.arguments.len() {
                        arguments.push(parse_math_part(node, chars, &mut i, context)?);
                    }

                    let instantiated = instantiate_tag(op, arguments);

                    let new_child_id = node.children.len();
                    node.children.push(instantiated);
                    res.push(NodeContent::Child(new_child_id));
                }
                else {
                    res.push(NodeContent::Character(c));
                }
            },
            NodeContent::Child(c) => {
                res.push(NodeContent::Child(c));
            },
        }

        i += 1;
    }

    node.content = res;

    return Ok(());
}


fn parse_math_part(node: &mut Node, chars: &Vec<char>, pos: &mut usize, context: &ParserContext) -> Result<Node, ParseError> {
    todo!(); // TODO
}


/// Tries to read an operator AFTER the question mark
fn expect_operator<'a>(node: &Node, chars: &Vec<char>, pos: &mut usize, context: &'a ParserContext) -> Result<&'a CustomTag, ParseError> {
    let mut word = String::with_capacity(15);
    let start_pos = *pos;

    loop {
        let el = &node.content[*pos];

        match *el {
            NodeContent::Character(c) => {
                if super::WORD_CHARS.contains(c) || c.is_alphabetic() {
                    word.push(c);
                }
                else {
                    break;
                }
            },
            NodeContent::Child(_) => {
                break;
            },
        }

        *pos += 1;
    }

    match context.math_operators.get(&word) {
        Some(op) => return Ok(op),
        None => 
            Err(ParseError { 
                message: format!("Unknown math operator name \"{}\"", word), 
                position: get_file_pos_of_char_in_node(node, chars, start_pos), 
                length: word.len() 
            }),
    }

}


/// Returns the proper error if a tag is present instead of a character
fn expect_character(node: &Node, chars: &Vec<char>, id: usize) -> Result<char, ParseError> {
    match node.content[id] {
        NodeContent::Character(c) => return Ok(c),
        _ => {
            let err_pos = get_file_pos_of_char_in_node(node, chars, id);
            return Err(ParseError { message: String::from("Didn't expected a tag here."), position: err_pos, length: 1 });
        }
    }
}


/// Get the file position of a character of a node. It's slow, use only for error reporting
fn get_file_pos_of_char_in_node(node: &Node, chars: &Vec<char>, id: usize) -> FilePosition {
    let mut res = node.start_inner_position.clone();
    
    for i in 0..id {
        match node.content[i] {
            NodeContent::Character(_) => advance_position(&mut res, chars),
            NodeContent::Child(c) =>  {
                for _ in 0..(node.children[c].source_length) {
                    advance_position(&mut res, chars);
                }
            },
        }
    }

    return res;
}


