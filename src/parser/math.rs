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


struct Alias {
    alias: &'static str,
    tag_name: &'static str,
    is_infix: bool,
}

macro_rules! alias {
    ($alias: literal, $tag_name: literal, $infix: expr) => {
        Alias { alias: $alias, tag_name: $tag_name, is_infix: $infix }
    };
}


/// HashMap of all aliases, each character maps to th corresponding default custom tag name 
static ALIASES: [Alias; 22] = [
    alias!("=", "equal", false),
    alias!(",", "comma", false),
    alias!("/", "frac", true),
    alias!("v/", "sqrt", false),
    alias!("+", "plus", false),
    alias!("-", "minus", false),
    alias!("€", "belongsto", false),
    alias!("^", "exponent", true),
    alias!("_", "subscript", true),
    alias!("^^", "overset", true),
    alias!("__", "underset", true),
    alias!("|", "normalfont", false),
    alias!("->", "rightarrow", false),
    alias!("=>", "rightdoublearrow", false),
    alias!("-->", "longrightarrow", false),
    alias!("==>", "longrightdoublearrow", false),
    alias!("<-", "leftarrow", false),
    alias!("<=", "leftdoublearrow", false),
    alias!("<--", "longleftarrow", false),
    alias!("<==", "longleftdoublearrow", false),
    alias!("<-->", "longleftrightarrow", false),
    alias!("<=>", "leftrightdoublearrow", false),
];


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

    let mut got_one_thing = false;

    while *index < node.content.len() {
        let c = &node.content[*index];

        // Look for aliases
        let alias = if context.ignore_aliases { None } else { check_for_alias(&node.content, *index) };
        match alias {
            Some(alias) => {
                let tag = context.math_operators.get(alias.tag_name);

                match tag {
                    Some(tag) => {
                        *index += alias.alias.chars().count();

                        let file_pos = get_file_pos_of_char_in_node_with_other_children(node, &children, chars, *index);

                        let mut arguments: Vec<Node> = Vec::new();
                        if alias.is_infix {
                            if tag.arguments.len() != 2 { // Invalid argument count
                                return Err(ParseError { 
                                    message: format!("Operator alias \"{}\" in infix but corresponds to the operator \"{}\" with {} arguments. This is probably because you have modified default.cowx.", alias.alias, alias.tag_name, tag.arguments.len()), 
                                    position: file_pos, length: alias.alias.len() 
                                })
                            }

                            // Nothing before infix operator
                            if res.len() == 0 {
                                return Err(ParseError { 
                                    message: format!("Expected something before \"{}\", because it's an infix operator. You should write {{left}}{}{{right}} instead of {}{{left}}{{right}}", alias.alias, alias.alias, alias.alias), 
                                    position: file_pos, length: alias.alias.len() 
                                });
                            }

                            // Gt the last element of the nodes's content
                            let left = match res.pop().unwrap() {
                                NodeContent::Character(c) |
                                NodeContent::EscapedCharacter(c) => Node { 
                                    name: String::from("div"), 
                                    attributes: vec![], 
                                    children: vec![], 
                                    content: vec![NodeContent::Character(c)], 
                                    auto_closing: false, 
                                    start_position: file_pos.clone(),
                                    start_inner_position: file_pos, 
                                    source_length: 1 
                                },
                                NodeContent::Child(_) => {
                                    res_children.pop().unwrap()
                                },
                            };

                            let right = parse_math_subgroup(node, children, chars, index, context, true)?;

                            arguments.push(left);
                            arguments.push(right);
                        }
                        else {
                            for _ in 0..tag.arguments.len() {
                                let child = parse_math_subgroup(node, children, chars, index, context, true)?;
                                arguments.push(child);
                            }
                        }

                        let instantiated = super::custom::instantiate_tag(tag, arguments);
                        let new_child_id = res_children.len();
                        res_children.push(instantiated);
                        res.push(NodeContent::Child(new_child_id));

                        got_one_thing = true;
                    },
                    None => return Err(ParseError { 
                        message: format!("Operator alias \"{}\" found, but corresponding tag \"{}\" not found. This is probably because you have modified default.cowx.", alias.alias, alias.tag_name), 
                        position: get_file_pos_of_char_in_node_with_other_children(node, &children, chars, *index), length: alias.alias.len() 
                    }),
                }
            },
            None => { // No alias found, check as a regular character 
                match *c {
                    NodeContent::Character(c) => {
                        if c == '?' { // Parse a math operator
                            *index += 1;
                            let op = expect_operator(node, chars, &children, index, context)?;
        
                            let mut arguments = Vec::with_capacity(op.arguments.len());
                            for _ in 0..op.arguments.len() {
                                let child = parse_math_subgroup(node, children, chars, index, context, true)?;
                                arguments.push(child);
                            }
        
                            let instantiated = super::custom::instantiate_tag(op, arguments);
        
                            let new_child_id = res_children.len();
                            res_children.push(instantiated);
                            res.push(NodeContent::Child(new_child_id));
                            got_one_thing = true;
                        }
                        else if c == '{' { // Sub group. Make a recursive call
                            *index += 1;
                            let child = parse_math_subgroup(node, children, chars, index, context, false)?;
        
                            let new_child_id = res_children.len();
                            res_children.push(child);
                            res.push(NodeContent::Child(new_child_id));
                            got_one_thing = true;
                        }
                        else if c == '}' { // Finished!
                            *index += 1;
                            break;
                        } 
                        else if c == '§' {
                            *index += 1;

                            let letter_to_convert = match node.content[*index] {
                                NodeContent::Character(l) => l,
                                NodeContent::EscapedCharacter(l) => {
                                    crate::log::warning_position(
                                        "Escaped character after \"§\". Consider removing the backslash.", 
                                        &get_file_pos_of_char_in_node_with_other_children(node, &children, chars, *index - 1), 
                                        3,
                                    );
                                    l
                                },
                                NodeContent::Child(_) => {
                                    return Err(ParseError {
                                        message: format!("Expected a character after \"§\", found a tag."),
                                        position: get_file_pos_of_char_in_node_with_other_children(node, &children, chars, *index - 1),
                                        length: 1,
                                    });
                                },
                            };

                            let greek_letter = letter_to_greek(letter_to_convert);
                            match greek_letter {
                                Some(l) => {
                                    res.push(NodeContent::Character(l));
                                    *index += 1;
                                    got_one_thing = true;
                                },
                                None => {
                                    return Err(ParseError {
                                        message: format!("Character \"{}\" after \"§\" does not correspond to a greek letter. Only a-z, A-Z are accepted, except for q, Q, w and W", letter_to_convert),
                                        position: get_file_pos_of_char_in_node_with_other_children(node, &children, chars, *index - 1),
                                        length: 2,
                                    });
                                },
                            }

                        }
                        else if c.is_whitespace() { // Ignore whitespace!
                            *index += 1;
                        }
                        else { // A normal character
                            res.push(NodeContent::Character(c));
                            *index += 1;
                            got_one_thing = true;
                        }
                    },
                    NodeContent::EscapedCharacter(c) => { // A normal character
                        res.push(NodeContent::Character(c));
                        *index += 1;
                        got_one_thing = true;
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

                        got_one_thing = true;
                    },
                }
            }
        };

        if got_one_thing && just_one_thing {
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

    while *pos < node.content.len() {
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


// OPTI: that's O(n²) because of chars().nth(). Also a lot of vec allocations
/// Returns the longest possible alias at specified position, returns None if no alias found 
fn check_for_alias(node_content: &Vec<NodeContent>, index: usize) -> Option<&'static Alias> {
    let mut potential_matchs: Vec<usize> = Vec::with_capacity(ALIASES.len());
    for i in 0..ALIASES.len() {
        potential_matchs.push(i);
    }

    let mut res = None;

    let mut pos = 0;
    loop {
        let mut new_vec = Vec::new();
        for i in potential_matchs.iter() {
            let opt_char = ALIASES[*i].alias.chars().nth(pos);

            match opt_char {
                Some(alias_char) => {
                    let node_char = match node_content[index + pos] {
                        NodeContent::Character(c) => c,
                        NodeContent::EscapedCharacter(_) => '\0',
                        NodeContent::Child(_) => '\0',
                    };
        
                    let still_ok = alias_char == node_char;
        
                    if still_ok {
                        new_vec.push(*i);
                    }
                },
                None => { // End of the alias
                    res = Some(&ALIASES[*i]); // Set the result
                },
            }

        }

        if new_vec.len() == 0 { // No more potential matches
            break;
        }

        potential_matchs = new_vec;
        pos += 1;
    }

    return res;
}


fn parse_math_subgroup(node: &mut Node, children: &mut Vec<PotentialChild>, chars: &Vec<char>, index: &mut usize, context: &ParserContext, just_one_thing: bool) -> Result<Node, ParseError> {
    let start_pos = *index;
    let start_pos_in_file = get_file_pos_of_char_in_node_with_other_children(node, &children, chars, start_pos);

    let partial_child = parse_math_part(node, children, chars, index, context, just_one_thing)?;

    let res = Node {
        name: String::from("div"),
        attributes: vec![],
        children: partial_child.children,
        content: partial_child.content,
        auto_closing: false,
        start_position: start_pos_in_file.clone(),
        start_inner_position: start_pos_in_file,
        source_length: *index - start_pos,
    };

    return Ok(res);
}


/// Converts a char to greek, returns None if non-alphabetical, Q or W
fn letter_to_greek(c: char) -> Option<char> {
    let ascii_code = c as u8;

    if !c.is_ascii() { return None; }

    if 'A' <= c && c <= 'Z' && c != 'Q' && c != 'W' {
        return Some("ΑΒΨΔΕΦΓΗΙΞΚΛΜΝΟΠ ΡΣΤΘΩ ΧΥΖ".chars().nth((ascii_code - ('A' as u8)) as usize).expect("Uuh?"));
    }
    else if 'a' <= c && c <= 'z' && c != 'q' && c != 'w' {
        return Some("αβψδεφγηιξκλμνοπ ρστθω χυζ".chars().nth((ascii_code - ('a' as u8)) as usize).expect("Uuh?"));
    }

    return None;
}

