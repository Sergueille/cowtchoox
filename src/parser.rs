
use std::vec::Vec;
use std::path::PathBuf;
use crate::util::FilePosition;

use self::custom::TagHash;

pub mod math;
pub mod custom;

// This file is parsing raw text into the Node struct

// TODO: support quotes around attribute
// TODO: copy the file position struct in th nodes, for later error reporting
// TODO: handle unexpected EOF (currently panics because accesses out of the bounds of the array)
// TODO: handle comments (currently it reads them as text)

// NOTE: currently auto-closing tags needs to include a / at the end (<br/>)
//       should-it be mandatory?

// NOTE: I think math parsing, and react-like tags will be implemented in another module, when reconstructing the document


/// Chars to make a word (tag name, attribute, ...). Alphanumeric characters also included. 
const WORD_CHARS: &str = "_-"; 

/// Attribute nam usd to indicate that a th tag has a "?" whn parsed.
/// FIXME: th user can declare it as a real attribute and break everything!
const MATH_OPERATOR_ATTRIB_NAME: &'static str = "math-operator";


#[derive(Debug, Clone)]
pub enum NodeContent {
    Character(char),
    EscapedCharacter(char), // Character with backslash before it
    Child(usize), // The positon of the child in the child array
}


/// This struct will own all of his children.
/// The content field is a vect of NodeContent. 
///     -> Each element is either a character or a child node. If it's a child, it indicates the position of the child in the children vector
#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Node>,
    pub content: Vec<NodeContent>,
    pub auto_closing: bool,
    pub start_position: FilePosition, // Where it is located in the source file
    pub start_inner_position: FilePosition, // Where  =the inner content is located in the source file
    pub source_length: usize, // How long it is in th source file
}


/// Describes what went wrong
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub position: FilePosition,
    pub length: usize,
}


/// Contains useful information to parse a document
pub struct ParserContext<'a> {
    pub args: &'a crate::Args, // Command line arguments
    pub math_operators: TagHash,
}


/// Parses a raw file.
/// 
/// # Arguments
/// * `file`: the raw contents of the file
/// 
/// # Returns
/// * the parsed node
/// 
pub fn parse_file(file_path: &PathBuf, chars: &Vec<char>, context: &ParserContext) -> Result<Node, ParseError> {
    return parse_tag(chars, &mut get_start_of_file_position(file_path.clone()), false, false, context);
}


/// Parses a part of the raw file. The beginning must be at the start of a node.
/// 
/// # Arguments
/// * `file`: the raw contents of the file
/// * `pos`: the index of the character where the function should start parsing
/// * `accept_question_mark`: is <?tag_name> allowed? Used for math operators declarations (if one is found, it's the attribute MATH_OPERATOR_ATTRIB_NAME is set)
/// 
/// # Returns
/// * the parsed node
/// 
pub fn parse_tag(chars: &Vec<char>, mut pos: &mut FilePosition, accept_question_mark: bool, math: bool, context: &ParserContext) -> Result<Node, ParseError> {
    expect(chars, &mut pos, '<')?;

    let start_pos = pos.clone();

    let has_question_mark = match expect(chars, pos, '?') {
        Ok(_) => true,
        Err(_) => false,
    };

    if !accept_question_mark && has_question_mark { // "?" not allowed
        return Err(ParseError { message: String::from("Expected tag name. \"?\" is not allowed here."), position: pos.clone(), length: 1 });
    }

    let mut attributes = Vec::with_capacity(10);

    if has_question_mark {
        attributes.push((String::from(MATH_OPERATOR_ATTRIB_NAME), String::from("")));
    }


    // Read tag name
    let tag_name = read_word(chars, &mut pos);

    // Read the attributes
    loop {
        let attr = read_word(chars, &mut pos);

        // No more attributes
        if attr == "" {
            break;
        }
        
        // Check for a =
        let exp = expect(chars, pos, '=');
        match exp {
            Ok(()) => { // There is a value: read it
                let value = read_word(chars, pos);
                attributes.push((attr, value));
            },
            Err(_) => { // No value
                attributes.push((attr, String::new()));
            },
        }
    }

    // Is auto-closing tag?
    let exp = expect(chars, pos, '/');

    // Are we closing the tag correctly?
    expect(chars, pos, '>')?;

    let inner_start_pos = pos.clone();

    match exp {
        Ok(()) => { // Auto-closing
            // Return the node directly
            let length = get_positions_difference(pos, &start_pos);
            return Ok(Node {
                name: tag_name,
                attributes,
                children: vec![],
                content: vec![],
                auto_closing: true,
                start_position: start_pos,
                start_inner_position: inner_start_pos,
                source_length: length,
            });
        },
        Err(_) => { 
            // Normal tag, continue execution
        },
    }

    let mut res = Node {
        name: tag_name,
        attributes,
        children: vec![],
        content: vec![],
        auto_closing: false,
        start_position: start_pos,
        start_inner_position: inner_start_pos,
        source_length: 0,
    };

    parse_inner_tag(chars, &mut res, pos, math || has_question_mark, context)?;
    advance_position(pos, chars);
    advance_position(pos, chars);
    
    // Parse the node's contents
    // Got out of the contents, now cursor is in closing tag
    let closing_tag_name = lookahead_word(chars, pos);
    if closing_tag_name != res.name {
        return Err(ParseError { 
            message: format!("Unmatched tag. Expected to close tag \"{}\", but found tag \"{}\"", res.name, closing_tag_name), 
            position: pos.clone(), 
            length: closing_tag_name.len() 
        });
    }
    read_word(chars, pos); // Advance cursor to after the tag name 

    // Check for the very last character...
    expect(chars, pos, '>')?;

    res.source_length = get_positions_difference(pos, &res.start_position);

    // Yay, the user gave a completely valid node!

    return Ok(res);
}


/// Parses text inside a tag
fn parse_inner_tag<'a>(chars: &Vec<char>, node: &'a mut Node, pos: &mut FilePosition, math: bool, context: &ParserContext) -> Result<(), ParseError> {
    let mut children: Vec<Node> = Vec::with_capacity(10);
    let mut content: Vec<NodeContent> = Vec::with_capacity(100);
    
    let mut backslashed_character = false; // Should the next character be ignored because of a backslash

    loop {
        let next = chars[(*pos).absolute_position];

        if backslashed_character { // Escaped by backslash
            content.push(NodeContent::EscapedCharacter(next));
            advance_position(pos, chars);
            backslashed_character = false;
            continue;
        }

        if next == '\\' { // Next char isn't a command
            backslashed_character = true;
            advance_position(pos, chars);
        }
        else if next == '<' { // Opening a new tag?
            match chars[(*pos).absolute_position + 1] {
                '/' => { // Actually, it's finished!
                    break;
                },
                '!' => { // It's a comment
                    advance_position(pos, chars);
                    // TODO: handle that
                },
                _ => { // It's a child
                    let child = parse_tag(chars, pos, false, math, context)?;

                    children.push(child);
                    content.push(NodeContent::Child(children.len() - 1));
                }
            }
        }
        else if next == '$' {
            advance_position(pos, chars);

            if !math { // Inner math tag
                let mut start_inner_position = pos.clone();
                advance_position(&mut start_inner_position, chars);

                let mut math_tag = Node {
                    name: String::from("math"),
                    attributes: vec![],
                    children: vec![],
                    content: vec![],
                    auto_closing: false,
                    start_position: pos.clone(),
                    start_inner_position,
                    source_length: 0
                };

                parse_inner_tag(chars, &mut math_tag, pos, true, context)?;

                math_tag.source_length = get_positions_difference(&pos, &math_tag.start_position);

                children.push(math_tag);
                content.push(NodeContent::Child(children.len() - 1));
            }
            else { // Finished math
                break;
            }
        }
        else if next.is_whitespace() {
            if !math { // Ignore whitespace in math
                match content.last() {
                    Some(NodeContent::Child(_)) => {
                        // Ignore
                    },
                    Some(NodeContent::Character(c)) | Some(NodeContent::EscapedCharacter(c)) => {
                        // Ignore if last chars is already whitespace
                        if !c.is_whitespace() {
                            // Add a space
                            content.push(NodeContent::Character(' '));
                        }
                    },
                    None => {
                        // Ignore
                    }
                }
            }
            
            advance_position(pos, chars);
        }
        else {
            // Add character
            content.push(NodeContent::Character(next));
            advance_position(pos, chars);
        }
    }

    node.children = children;
    node.content = content;

    // Now that the tag is parsed, if it's math, parse math
    if math {
        math::parse_math(node, chars, context)?;
    }

    return Ok(());
}


/// Returns the inner content's inner text, ignoring inner tags
pub fn get_node_content_as_str(node: &Node) -> String {
    let mut res = String::with_capacity(node.content.len() - node.children.len());

    for cont in &node.content {
        match cont {
            NodeContent::Character(c) => res.push(*c),
            _ => {},
        }
    }

    return res;
}


/// Gets the valu of an attribute of a node. If doesn't exists, returns Err(). If it does exists but has no value, returns Ok(())
pub fn get_attribute_value<'a>(node: &'a Node, attrib_name: &str) -> Result<&'a String, ()> {
    for (name, val) in &node.attributes {
        if attrib_name == name {
            return Ok(&val);
        }
    }

    return Err(());
}


/// Advances the cursor until non-whitespace is found, then returns an error if the specified character isn't found
fn expect(chars: &Vec<char>, pos: &mut FilePosition, char: char) -> Result<(), ParseError> {
    advance_until_non_whitespace(chars, pos);

    if chars[(*pos).absolute_position] != char {
        return Err(ParseError { message: format!("Expected {}.", char), position: pos.clone(), length: 1 })
    }

    advance_position(pos, chars);

    return Ok(());
}


/// Advances the cursor until non-whitespace is found. The cursor will be on the first non-whitespace character.
/// If EOF is found, is places the cursor at the end of the file
fn advance_until_non_whitespace(chars: &Vec<char>, pos: &mut FilePosition) {
    while (*pos).absolute_position < chars.len() && chars[(*pos).absolute_position].is_whitespace() {
        advance_position(pos, chars);
    }
}


/// Reads a word and moves the cursor (case insensitive, return lowered chars!)
fn read_word(chars: &Vec<char>, pos: &mut FilePosition) -> String {
   advance_until_non_whitespace(chars, pos);
    let mut res = Vec::with_capacity(10);

    while chars[(*pos).absolute_position].is_alphanumeric() || WORD_CHARS.contains(chars[(*pos).absolute_position]) {
        res.push(chars[(*pos).absolute_position]);
        advance_position(pos, chars);
    }

    return res.into_iter().collect::<String>().to_lowercase();
}


/// Reads a word without moving the cursor
fn lookahead_word(chars: &Vec<char>, pos: &mut FilePosition) -> String {
    return read_word(chars, &mut pos.clone());
}


/// Advances a position, the character is used to take new line into account
pub fn advance_position(pos: &mut FilePosition, file: &Vec<char>) {
    (*pos).absolute_position += 1;
    (*pos).line_character += 1;
    
    let character = file[pos.absolute_position - 1];

    // FIXME: will not count lines for os that use \r
    if character == '\n' {
        (*pos).line += 1;
        (*pos).line_character = 0;
    }
}


fn get_positions_difference(a: &FilePosition, b: &FilePosition) -> usize {
    if a.file_path != b.file_path {
        panic!("Tried to compare positions located in different files");
    }

    return a.absolute_position - b.absolute_position;
}


/// basically call advance_position `count` times
pub fn advance_position_many(pos: &mut FilePosition, file: &Vec<char>, count: usize) {
    for _ in 0..count {
        advance_position(pos, file);
    }
}


/// Get a file position at the beginning of the file with given path
pub fn get_start_of_file_position(path: PathBuf) -> FilePosition {
    return FilePosition {   
        file_path: std::rc::Rc::from(path),
        absolute_position: 0,
        line: 0,
        line_character: 0
    };
}


/// Get the file position of a character of a node. It's slow, use only for error reporting
fn get_file_pos_of_char_in_node(node: &Node, chars: &Vec<char>, id: usize) -> FilePosition {
    let mut res = node.start_inner_position.clone();
    
    for i in 0..id {
        match node.content[i] {
            NodeContent::Character(_) => advance_position(&mut res, chars),
            NodeContent::EscapedCharacter(_) => {  // Advance twice. For the backslash AND the escaped character
                advance_position(&mut res, chars);
                advance_position(&mut res, chars);
            },
            NodeContent::Child(c) =>  {
                for _ in 0..(node.children[c].source_length) {
                    advance_position(&mut res, chars);
                }
            },
        }
    }

    return res;
}


