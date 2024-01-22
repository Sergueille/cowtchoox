
use std::vec::Vec;
use std::path::PathBuf;

// This file is parsing raw text into the Node struct

// TODO: support quotes around attributes
// TODO: support \
// TODO: copy the file position struct in th nodes, for later error reporting
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
    pub auto_closing: bool,
}


/// Describes what went wrong
#[derive(Debug)]
pub enum ParseError {
    Expected(String, FilePosition),
    UnmatchedTag(String, FilePosition),
}


/// Indicates a position in a file
/// All fields start at 0. Even lines.
#[derive(Clone, Debug)]
pub struct FilePosition {
    pub file_path: PathBuf,
    pub absolute_position: usize,
    pub line: usize,
    pub line_character: usize,
}


/// Parses a raw file.
/// 
/// # Arguments
/// * `file`: the raw contents of the file
/// 
/// # Returns
/// * the parsed node
/// 
pub fn parse_file(file_path: &PathBuf, chars: &Vec<char>) -> Result<Node, ParseError> {
    return parse_file_part(chars, &mut FilePosition {
        file_path: file_path.clone(),
        absolute_position: 0,
        line: 0,
        line_character: 0
    });
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
pub fn parse_file_part(chars: &Vec<char>, mut pos: &mut FilePosition) -> Result<Node, ParseError> {
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
                auto_closing: true,
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
        let next = chars[(*pos).absolute_position];

        // Opening a new tag?
        if next == '<' {
            match chars[(*pos).absolute_position + 1] {
                '/' => { // Actually, it's finished!
                    advance_position(pos, chars);
                    advance_position(pos, chars);
                    break;
                },
                '!' => { // It's a comment
                    advance_position(pos, chars);
                    // TODO: handle that
                },
                _ => { // It's a child
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

        advance_position(pos, chars);
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
        auto_closing: false,
    };

    return Ok(res);
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


/// Advances the cursor until non-whitespace is found, then returns an error if the specified character isn't found
fn expect(chars: &Vec<char>, pos: &mut FilePosition, char: char) -> Result<(), ParseError> {
    advance_until_non_whitespace(chars, pos);

    if chars[(*pos).absolute_position] != char {
        return Err(ParseError::Expected(char.to_string(), pos.clone()));
    }

    advance_position(pos, chars);

    return Ok(());
}


/// Advances the cursor until non-whitespace is found. The cursor will be on the first non-whitespace character
fn advance_until_non_whitespace(chars: &Vec<char>, pos: &mut FilePosition) {
    while chars[(*pos).absolute_position].is_whitespace() {
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
fn advance_position(pos: &mut FilePosition, file: &Vec<char>) {
    (*pos).absolute_position += 1;
    (*pos).line_character += 1;
    
    let character = file[pos.absolute_position - 1];

    // FIXME: will increment the line number by 2 on windows. fuck microsoft
    if character == '\n' || character == '\r' {
        (*pos).line += 1;
        (*pos).line_character = 0;
    }
}

