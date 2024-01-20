
use std::vec::Vec;

// This file is parsing raw text into the Node struct

// TODO: support quotes around attributes
// TODO: support \
// TODO: pass a struct with position ifo that include {pos in file, line, etc.} instead of just the index of the char (useful for error reporting)
// TODO: handle unexpected EOF (currently panics because accesses out of the bounds of the array)
// TODO: check for $
// TODO: handle comments (currently it reads them as text)

// NOTE: currently auto-closing tags needs to include a / at the end (<br/>)
//       should-it be mandatory?

// NOTE: I think math parsing, and react-like tags will be implemented in another module, when reconstructing the document


const WORD_CHARS: &str = "_-"; // Chars to make a word (tag name, attribute, ...). Alphanumeric characters also included. 


#[derive(Debug)]
pub enum NodeContent {
    Character(char),
    Child(usize), // The positon of the child in the child array
}


/// This struct will own all of his children, their position is indicated by the "content" field
#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Node>,
    pub content: Vec<NodeContent>,
}


/// Describes what went wrong
#[derive(Debug)]
pub enum ParseError {
    Expected(String, usize),
    UnmatchedTag(String, usize),
}


/// Parses a raw file.
/// 
/// # Arguments
/// * `file`: the raw contents of the file
/// 
/// # Returns
/// * the parsed node
/// 
pub fn parse_file(chars: &Vec<char>) -> Result<Node, ParseError> {
    return parse_file_part(chars, &mut 0);
}


/// Parses a part of the raw file. The beginning must be at the start of a node.
/// 
/// # Arguments
/// * `file`: the raw contents of the file
/// * `pos`: the index of the character where the function should start parsing
/// 
/// # Returns
/// * the parsed node
/// 
pub fn parse_file_part(chars: &Vec<char>, mut pos: &mut usize) -> Result<Node, ParseError> {
    expect(chars, &mut pos, '<')?;

    // Read tag name
    let tag_name = read_word(chars, &mut pos);

    let mut attributes = Vec::with_capacity(10);

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

    match exp {
        Ok(()) => { // Auto-closing
            // Return the node directly
            return Ok(Node {
                name: tag_name,
                attributes,
                children: vec![],
                content: vec![],
            });
        },
        Err(_) => { 
            // Normal tag, continue execution
        },
    }
    
    // Parse the node's contents
    let mut children: Vec<Node> = Vec::with_capacity(10);
    let mut content: Vec<NodeContent> = Vec::with_capacity(100);
    
    loop {
        let next = chars[*pos];

        // Opening a new tag?
        if next == '<' {
            *pos += 1;

            match chars[*pos] {
                '/' => { // Actually, it's finished!
                    *pos += 1;
                    break;
                },
                '!' => { // It's a comment
                    // TODO: handle that
                },
                _ => { // It's a child
                    *pos -= 1; // Get back one char, so that the recursive call can read the <
                    let child = parse_file_part(chars, pos)?;

                    children.push(child);
                    content.push(NodeContent::Child(children.len() - 1));
                }
            }
        }
        else if next.is_whitespace() {
            match content.last() {
                Some(NodeContent::Child(_)) => {
                    // Ignore
                },
                Some(NodeContent::Character(c)) => {
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
        else {
            // Add character
            content.push(NodeContent::Character(next));
        }

        *pos += 1;
    }

    // Got out of the contents, now cursor is in closing tag
    let closing_tag_name = read_word(chars, pos);
    if closing_tag_name != tag_name {
        return Err(ParseError::UnmatchedTag(tag_name, pos.clone()));
    }

    // Check for the very last character...
    expect(chars, pos, '>')?;

    // Yay, the user gave a completely valid node!
    let res = Node {
        name: tag_name,
        attributes,
        children,
        content,
    };

    return Ok(res);
}


/// Advances the cursor until non-whitespace is found, then returns an error if the specified character isn't found
fn expect(chars: &Vec<char>, pos: &mut usize, char: char) -> Result<(), ParseError> {
    advance_until_non_whitespace(chars, pos);

    if chars[*pos] != char {
        return Err(ParseError::Expected(char.to_string(), pos.clone()));
    }

    *pos += 1;

    return Ok(());
}


/// Advances the cursor until non-whitespace is found. The cursor will be on the first non-whitespace character
fn advance_until_non_whitespace(chars: &Vec<char>, pos: &mut usize) {
    while chars[*pos].is_whitespace() {
        *pos += 1;
    }
}


/// Reads a word and moves the cursor
fn read_word(chars: &Vec<char>, pos: &mut usize) -> String {
   advance_until_non_whitespace(chars, pos);
    let mut res = Vec::with_capacity(10);

    while chars[*pos].is_alphanumeric() || WORD_CHARS.contains(chars[*pos]) {
        res.push(chars[*pos]);
        *pos += 1;
    }

    return res.into_iter().collect();
}


/// Reads a word without moving the cursor
fn lookahead_word(chars: &Vec<char>, pos: &mut usize) -> String {
    return read_word(chars, &mut pos.clone());
}


