use super::{Node, NodeContent, ParseError, ParserContext, FilePosition};
use super::custom::CustomTag;

struct PartialNode {
    children: Vec<Node>,
    content: Vec<super::NodeContent>,
}

enum PotentialChild {
    Some(Node),
    None(usize) // Contains the source length
}


/// Create math! Called after tags are parsed. Will replace the provided Node's contents by math.
/// 
/// # Arguments
/// * `node`: A node. It's children are fully math-parsed, but not it's inner text
/// 
/// # Returns
/// The node that contains all the math.
/// 
pub fn parse_math(node: &mut Node, chars: &Vec<char>, context: &ParserContext) -> Result<(), ParseError> {
    let mut pos = 0;

    // Remove children from node to take ownership
    let raw_children = std::mem::replace(&mut node.children, vec![]);
    let mut children = raw_children.into_iter().map(|el| { PotentialChild::Some(el) }).collect();

    let res = parse_math_part(node, &mut children, chars, &mut pos, context, false)?;

    // Check for unmatched }
    if pos < node.content.len() {
        return Err(ParseError {
            message: String::from("Unexpected \"}\". Maybe you forgot to add the matching curly bracket."),
            position: get_file_pos_of_char_in_node_with_other_children(node, &children, chars, pos),
            length: 1,
        });
    }

    // Replace node's contents
    node.children = res.children;
    node.content = res.content;

    return Ok(()); 
}


/// Sub-function of parse_math. `pos` is the position in the node's content array
/// TODO: absolutely not finished
fn parse_math_part(node: &mut Node, children: &mut Vec<PotentialChild>, chars: &Vec<char>, index: &mut usize, context: &ParserContext, just_one_thing: bool) 
    -> Result<PartialNode, ParseError> {
    let mut res: Vec<NodeContent> = Vec::with_capacity(node.content.len());
    let mut res_children: Vec<Node> = Vec::with_capacity(5);

    while *index < node.content.len() {
        let c = &node.content[*index];

        match *c {
            NodeContent::Character(c) => {
                if c == '?' { // Parse a math operator
                    *index += 1;
                    let op = expect_operator(node, chars, &children, index, context)?;

                    let mut arguments = Vec::with_capacity(op.arguments.len());
                    for _ in 0..op.arguments.len() {
                        let start_pos = *index - 1;
                        let start_pos_in_file = get_file_pos_of_char_in_node_with_other_children(node, children, chars, start_pos);

                        let partial_child = parse_math_part(node, children, chars, index, context, true)?;

                        let child_node = Node {
                            name: String::from("div"),
                            attributes: vec![],
                            children: partial_child.children,
                            content: partial_child.content,
                            auto_closing: false,
                            start_position: start_pos_in_file.clone(),
                            start_inner_position: start_pos_in_file,
                            source_length: *index - start_pos,
                        };

                        arguments.push(child_node);
                    }

                    let instantiated = super::custom::instantiate_tag(op, arguments);

                    let new_child_id = res_children.len();
                    res_children.push(instantiated);
                    res.push(NodeContent::Child(new_child_id));
                }
                else if c == '{' { // Sub group. Make a recursive call
                    *index += 1;
                    let start_pos = *index;
                    let start_pos_in_file = get_file_pos_of_char_in_node_with_other_children(node, &children, chars, start_pos);

                    let partial_child = parse_math_part(node, children, chars, index, context, false)?;

                    let child_node = Node {
                        name: String::from("div"),
                        attributes: vec![],
                        children: partial_child.children,
                        content: partial_child.content,
                        auto_closing: false,
                        start_position: start_pos_in_file.clone(),
                        start_inner_position: start_pos_in_file,
                        source_length: *index - start_pos,
                    };

                    let new_child_id = res_children.len();
                    res_children.push(child_node);
                    res.push(NodeContent::Child(new_child_id));
                }
                else if c == '}' { // Finished!
                    *index += 1;
                    break;
                } 
                else if c.is_whitespace() { // Ignore whitespace!
                    *index += 1;
                }
                else { // A normal character
                    res.push(NodeContent::Character(c));
                    *index += 1;
                }
            },
            NodeContent::EscapedCharacter(c) => { // A normal character
                res.push(NodeContent::Character(c));
                *index += 1;
            }
            NodeContent::Child(c) => { // A child, just push it as a normal NodeContent
                let length = match &children[c] {
                    PotentialChild::Some(child) => child.source_length,
                    PotentialChild::None(_) => panic!("Should be Some"),
                };

                let child = std::mem::replace(&mut children[c], PotentialChild::None(length));

                match child {
                    PotentialChild::Some(child) =>{
                        res_children.push(child);
                        res.push(NodeContent::Child(res_children.len() - 1));
                        
                        *index += 1;
                    },
                    PotentialChild::None(_) => unreachable!(),
                }
            },
        }

        if just_one_thing {
            break;
        }
    }

    return Ok(PartialNode {
        children: res_children,
        content: res,
    });
}


/// Tries to read an operator AFTER the question mark
fn expect_operator<'a>(node: &Node, chars: &Vec<char>, children: &Vec<PotentialChild>, pos: &mut usize, context: &'a ParserContext) -> Result<&'a CustomTag, ParseError> {
    let mut word = String::with_capacity(15);
    let start_pos = *pos - 1;

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
            NodeContent::EscapedCharacter(_) => {
                break;
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
                position: get_file_pos_of_char_in_node_with_other_children(node, children, chars, start_pos), 
                length: word.len() + 1
            }),
    }

}


/// Returns the proper error if a tag is present instead of a character
fn expect_character(node: &Node, chars: &Vec<char>, children: &Vec<PotentialChild>, id: usize) -> Result<char, ParseError> {
    match node.content[id] {
        NodeContent::Character(c) => return Ok(c),
        _ => {
            let err_pos = get_file_pos_of_char_in_node_with_other_children(node, children, chars, id);
            return Err(ParseError { message: String::from("Didn't expected a tag here."), position: err_pos, length: 1 });
        }
    }
}


/// Same as parser::get_file_pos_of_char_in_node
fn get_file_pos_of_char_in_node_with_other_children(node: &Node, children: &Vec<PotentialChild>, chars: &Vec<char>, id: usize) -> FilePosition {
    let mut res = node.start_inner_position.clone();
    
    for i in 0..id {
        match node.content[i] {
            NodeContent::Character(_) => super::advance_position(&mut res, chars),
            NodeContent::EscapedCharacter(_) => {  // Advance twice. For the backslash AND the escaped character
                super::advance_position(&mut res, chars);
                super::advance_position(&mut res, chars);
            },
            NodeContent::Child(c) =>  {
                let source_length = match &children[c] {
                    PotentialChild::Some(child) => child.source_length,
                    PotentialChild::None(l) => *l,
                };

                 for _ in 0..source_length {
                    super::advance_position(&mut res, chars);
                }
            },
        }
    }

    return res;
}
