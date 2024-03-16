
use std::vec::Vec;
use std::path::PathBuf;

use crate::Context;
use crate::util::FilePosition;

pub mod math;
pub mod custom;

// This file is parsing raw text into the Node struct

// TODO: handle HTML comments

// NOTE: currently auto-closing tags needs to include a / at the end (<br/>)
//       should-it be mandatory?


/// Chars to make a word (tag name, attribute, ...). Alphanumeric characters also included. 
const WORD_CHARS: &str = "_-:"; 


#[derive(Debug, Clone)]
pub enum NodeContent {
    Character((char, FilePosition)), // Contains the char, then it's absolute position in the file
    EscapedCharacter((char, FilePosition)), // Character with backslash before it
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
    pub declaration_symbol: TagSymbol,
    pub start_position: FilePosition, // Where it is located in the source file
    pub start_inner_position: FilePosition, // Where the inner content is located in the source file
    pub source_length: usize, // How long it is in th source file
}


/// Describes what went wrong
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub position: FilePosition,
    pub length: usize,
}


/// Used internally to determine if inside math node or code block
#[derive(PartialEq, Eq, Debug)]
enum ParserState {
    Normal, Math, BigMath, Code, BigCode
}


bitflags::bitflags! {
    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    pub struct TagSymbol: u8 {
        const NOTHING = 0x1;
        const QUESTION_MARK = 0x2;
        const EXCLAMATION_MARK = 0x4;
    }
}
    

/// Parses a raw file.
/// 
/// # Arguments
/// * `file`: the raw contents of the file
/// 
/// # Returns
/// * the parsed node
/// 
pub fn parse_file(file_path: &PathBuf, chars: &Vec<char>, context: &Context) -> Result<Node, ParseError> {
    return parse_tag(chars, &mut get_start_of_file_position(file_path.clone()), TagSymbol::NOTHING, false, false, context);
}


/// Parses a part of the raw file. The beginning must be at the start of a node.
/// 
/// # Arguments
/// * `file`: the raw contents of the file
/// * `pos`: the index of the character where the function should start parsing
/// * `expect_symbol`: the expected thing after the first `<`. For nothing use `TagSymbol::Nothing` 
/// * `is_not_custom_tag`: if true, will not interpret it as a custom tag, event if a `!` is present 
/// 
/// # Returns
/// * the parsed node
/// 
pub fn parse_tag(chars: &Vec<char>, mut pos: &mut FilePosition, expect_symbol: TagSymbol, math: bool, is_not_custom_tag: bool, context: &Context) -> Result<Node, ParseError> {
    expect(chars, &mut pos, '<')?;

    let start_pos = pos.clone();

    let next_char = read_next_char(chars, pos)?;
    let used_symbol = match next_char {
        '?' => TagSymbol::QUESTION_MARK,
        '!' => TagSymbol::EXCLAMATION_MARK,
        _ => {
            *pos = start_pos.clone(); // Get back to last position, because we advanced past the beginning of the tag names
            TagSymbol::NOTHING
        },
    };

    // The thing we got does not correspond to what is expected
    if (used_symbol & expect_symbol).bits() == 0 {
        if used_symbol == TagSymbol::NOTHING {
            return Err(ParseError {
                message: String::from("Expected something here."),
                position: start_pos,
                length: 2,
            });
        }
        else {
            return Err(ParseError {
                message: format!("Unexpected \"{}\"", next_char),
                position: start_pos,
                length: 2,
            });
        }
    }

    let mut attributes = Vec::with_capacity(10);

    // Read tag name
    let tag_name = read_word(chars, &mut pos)?;

    // Read the attributes
    loop {
        let attr = read_word(chars, &mut pos)?;

        // No more attributes
        if attr == "" {
            break;
        }
        
        // Check for a =
        let exp = expect(chars, pos, '=');
        match exp {
            Ok(()) => { // There is a value: read it
                advance_until_non_whitespace(chars, pos)?;
                let use_quotes = chars[pos.absolute_position] == '"';

                let value;
                if use_quotes {
                    advance_position_with_comments(pos, chars)?;
                    value = read_until_quote(chars, pos)?;
                }
                else {
                    value = read_word(chars, pos)?;
                }

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

    let mut res;

    match exp {
        Ok(()) => { // Auto-closing
            // Return the node directly
            let length = get_positions_difference(pos, &start_pos);
            res = Node {
                name: tag_name,
                attributes,
                children: vec![],
                content: vec![],
                auto_closing: true,
                declaration_symbol: used_symbol,
                start_position: start_pos,
                start_inner_position: inner_start_pos,
                source_length: length,
            };
        },
        Err(_) => { // Not auto-closing
            let is_really_math = math || used_symbol == TagSymbol::QUESTION_MARK || tag_name == "mathnode";

            res = Node {
                name: tag_name,
                attributes,
                children: vec![],
                content: vec![],
                auto_closing: false,
                declaration_symbol: used_symbol,
                start_position: start_pos,
                start_inner_position: inner_start_pos,
                source_length: 0,
            };

            parse_inner_tag(chars, &mut res, pos, if is_really_math { ParserState::Math } else { ParserState::Normal }, context)?;
            advance_position(pos, chars)?;
            advance_position(pos, chars)?;
            
            // Parse the node's contents
            // Got out of the contents, now cursor is in closing tag
            let closing_tag_name = lookahead_word(chars, pos)?;
            if closing_tag_name != res.name {
                return Err(ParseError { 
                    message: format!("Unmatched tag. Expected to close tag \"{}\", but found tag \"{}\"", res.name, closing_tag_name), 
                    position: pos.clone(), 
                    length: closing_tag_name.len() 
                });
            }
            read_word(chars, pos)?; // Advance cursor to after the tag name 

            // Check for the very last character...
            expect(chars, pos, '>')?;

            res.source_length = get_positions_difference(pos, &res.start_position);
        },
    }

    // Yay, the user gave a completely valid node!

    // Now, if it's a custom tag, instantiate it properly
    if !is_not_custom_tag && res.declaration_symbol == TagSymbol::EXCLAMATION_MARK  {
        let custom_tag = match context.custom_tags.get(&res.name) {
            Some(tag) => tag,
            None => {
                return Err(ParseError {
                    message: format!("Unknown custom tag \"{}\" used", res.name),
                    position: res.start_position,
                    length: res.name.len() + 1,
                });
            }
        };

        if custom_tag.is_math {
            return Err(ParseError {
                message: format!("You tried to use \"{}\" as a custom tag, but it has been declared as a math operator. Use it with the math operator syntax.", res.name),
                position: res.start_position,
                length: res.name.len() + 1,
            });  
        }

        let mut arguments = Vec::with_capacity(res.attributes.len() + 1);
        for (name, value) in res.attributes.iter() {
            let mut chars = name.chars();
            if chars.next().unwrap() == ':' {
                arguments.push((chars.collect(), get_tag_from_raw_text(value.as_str(), pos)));
            } 
        }

        let start_position = res.start_position.clone();

        let has_inner = custom::has_inner_param(custom_tag);
        if res.auto_closing {
            if has_inner {
                return Err(ParseError {
                    message: format!("The custom tag \"{}\" should not be auto-closing. You should usee it like this: \"<!{}></{}>\".", res.name, res.name, res.name),
                    position: res.start_position,
                    length: res.name.len() + 1,
                });  
            }
        }
        else {
            if !has_inner {
                return Err(ParseError {
                    message: format!("The custom tag \"{}\" should be auto-closing. You should usee it like this: \"<!{}/>\".", res.name, res.name),
                    position: res.start_position.clone(),
                    length: res.name.len() + 1,
                });  
            }

            arguments.push((String::from("inner"), res)); // Push the inner content as an ":inner" argument
        }

        let actual_res = custom::instantiate_tag_with_named_parameters(custom_tag, arguments, &start_position)?;

        return Ok(actual_res);
    }
    else {
        return Ok(res);
    }
}


/// Parses text inside a tag
fn parse_inner_tag<'a>(chars: &Vec<char>, node: &'a mut Node, pos: &mut FilePosition, state: ParserState, context: &Context) -> Result<(), ParseError> {
    let mut children: Vec<Node> = Vec::with_capacity(10);
    let mut content: Vec<NodeContent> = Vec::with_capacity(100);
    
    let mut backslashed_character = false; // Should the next character be ignored because of a backslash

    loop {
        let next = chars[(*pos).absolute_position];

        // Code: ignore all but backticks
        if state == ParserState::Code || state == ParserState::BigCode {
            if next == '`' {
                advance_position_with_comments(pos, chars)?;
            
                let double = chars[pos.absolute_position] == '`';

                if state == ParserState::Code {
                    if double {
                        return Err(ParseError {
                            message: String::from("Expected a single backtick (and not two) to close code block."),
                            position: pos.clone(),
                            length: 1,
                        })
                    }
                    else {
                        break;
                    }
                }
                else if state == ParserState::BigCode {
                    if double {
                        advance_position_with_comments(pos, chars)?;
                        break;
                    }
                    else {
                        //It's a regular character
                        content.push(NodeContent::Character(('`', pos.clone())));
                    }
                }
            }
            else {
                advance_position_with_comments(pos, chars)?;
                content.push(NodeContent::Character((next, pos.clone())));
            }

            continue; // Nothing else!
        }

        // If not code:

        if backslashed_character { // Escaped by backslash
            content.push(NodeContent::EscapedCharacter((next, pos.clone())));
            advance_position(pos, chars)?;
            backslashed_character = false;
            continue;
        }

        if next == '\\' { // Next char isn't a command
            backslashed_character = true;
            advance_position(pos, chars)?;
        }
        else if next == '<' { // Opening a new tag?
            match chars[(*pos).absolute_position + 1] {
                '/' => { // Actually, it's finished!
                    break;
                },
                _ => { // It's a child
                    let mut res_pos = pos.clone();
                    let result = parse_tag(
                        chars, 
                        &mut res_pos, 
                        TagSymbol::NOTHING | TagSymbol::EXCLAMATION_MARK, 
                        state == ParserState::Math || state == ParserState::BigMath, 
                        false, 
                        context
                    );

                    match result {
                        Ok(child) => {
                            children.push(child);
                            content.push(NodeContent::Child(children.len() - 1));
                            *pos = res_pos;
                        }
                        Err(e) => { // Didn't work! Maybe because in math some characters looks like tags but aren't
                            if state == ParserState::Math { // If error, just interpret as regular text
                                content.push(NodeContent::Character(('<', pos.clone())));
                                advance_position(pos, chars)?;
                            }
                            else {
                                return Err(e);
                            }
                        },
                    }

                }
            }
        }
        else if next == '$' {
            let pos_before_dollar = pos.clone();
            advance_position(pos, chars)?;

            let double = chars[pos.absolute_position] == '$';
            if double {
                advance_position(pos, chars)?;
            }

            if state == ParserState::Normal { // Inner math tag
                let mut start_inner_position = pos.clone();
                advance_position(&mut start_inner_position, chars)?;

                let attributes = if double {
                    vec![(String::from("class"), String::from("center"))]
                }
                else {
                    vec![]
                };

                let mut math_tag = Node {
                    name: String::from("math"),
                    attributes,
                    children: vec![],
                    content: vec![],
                    auto_closing: false,
                    declaration_symbol: TagSymbol::NOTHING,
                    start_position: pos.clone(),
                    start_inner_position,
                    source_length: 0
                };

                let math_type = if double { ParserState::BigMath } else { ParserState::Math }; 

                parse_inner_tag(chars, &mut math_tag, pos, math_type, context)?;

                math_tag.source_length = get_positions_difference(&pos, &math_tag.start_position);

                children.push(math_tag);
                content.push(NodeContent::Child(children.len() - 1));
            }
            else if state == ParserState::Math { // Finished math
                if double {
                    return Err(ParseError {
                        message: String::from("Closing inline math with \"$$\" but it started with\"$\". Consider removing one dollar, or putting a space between them."),
                        position: pos_before_dollar,
                        length: 2,
                    });
                }

                break;
            }
            else if state == ParserState::BigMath { // Finished big math
                if !double {
                    return Err(ParseError {
                        message: String::from("Closing math with \"$\", but it started with \"$$\". Consider adding one extra dollar."),
                        position: pos_before_dollar,
                        length: 1,
                    });
                }

                break;
            }
            else { unreachable!() }
        }
        else if next == '`' && state != ParserState::Math && state != ParserState::BigMath { // Code block
            advance_position_with_comments(pos, chars)?;
            
            let double = chars[(*pos).absolute_position] == '`';

            let mut start_inner_position = pos.clone();
            advance_position_with_comments(&mut start_inner_position, chars)?;

            // Make big code if two backticks
            let tag_name = if double {
                String::from("pre")
            }
            else {
                String::from("code")
            };

            let mut math_tag = Node {
                name: tag_name,
                attributes: vec![],
                children: vec![],
                content: vec![],
                auto_closing: false,
                declaration_symbol: TagSymbol::NOTHING,
                start_position: pos.clone(),
                start_inner_position,
                source_length: 0
            };
            
            if double {
                advance_position_with_comments(pos, chars)?;
            } 

            parse_inner_tag(chars, &mut math_tag, pos, if double { ParserState::BigCode } else { ParserState::Code }, context)?;


            math_tag.source_length = get_positions_difference(&pos, &math_tag.start_position);

            children.push(math_tag);
            content.push(NodeContent::Child(children.len() - 1));
        }
        else if next.is_whitespace() {
            match content.last() {
                Some(NodeContent::Child(_)) => {
                    // Add a space
                    content.push(NodeContent::Character((' ', pos.clone())));
                },
                Some(NodeContent::Character((c, _))) | Some(NodeContent::EscapedCharacter((c, _))) => {
                    // Ignore if last chars is already whitespace
                    if !c.is_whitespace() {
                        // Add a space
                        content.push(NodeContent::Character((' ', pos.clone())));
                    }
                },
                None => {
                    // Ignore
                }
            }
            
            advance_position(pos, chars)?;
        }
        else {
            // Add character
            content.push(NodeContent::Character((next, pos.clone())));
            advance_position(pos, chars)?;
        }
    }

    node.children = children;
    node.content = content;

    // Now that the tag is parsed, if it's math, parse math
    if state == ParserState::Math || state == ParserState::BigMath {
        math::parse_math(node, context)?;
    }

    return Ok(());
}


/// Returns the inner content's inner text, ignoring inner tags
pub fn get_node_content_as_str(node: &Node) -> String {
    let mut res = String::with_capacity(node.content.len() - node.children.len());

    for cont in &node.content {
        match cont {
            NodeContent::Character((c, _)) | NodeContent::EscapedCharacter((c, _)) => res.push(*c),
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


/// Advances the cursor until non-whitespace is found (or end of file), then returns an error if the specified character isn't found
fn expect(chars: &Vec<char>, pos: &mut FilePosition, char: char) -> Result<(), ParseError> {
    advance_until_non_whitespace(chars, pos)?;

    if (*pos).absolute_position >= chars.len() {
        return Err(ParseError { 
            message: format!("Expected \"{}\", found end of file.", char), 
            position: pos.clone(), 
            length: 1 
        });
    }

    if chars[(*pos).absolute_position] != char {
        return Err(ParseError { 
            message: format!("Expected \"{}\", found \"{}\".", char, chars[(*pos).absolute_position]), 
            position: pos.clone(), 
            length: 1 
        });
    }

    advance_position(pos, chars)?;

    return Ok(());
}



/// Advances the cursor until non-whitespace is found (or end of file), then returns an error if the specified character isn't found
fn read_next_char(chars: &Vec<char>, pos: &mut FilePosition) -> Result<char, ParseError> {
    advance_until_non_whitespace(chars, pos)?;

    if (*pos).absolute_position >= chars.len() {
        return Err(ParseError { 
            message: String::from("Expected something, found end of file."), 
            position: pos.clone(), 
            length: 1 
        });
    }

    let res = chars[(*pos).absolute_position];

    advance_position(pos, chars)?;
    return Ok(res);
}


/// Advances the cursor until non-whitespace is found. The cursor will be on the first non-whitespace character.
/// If already on a non-whitespace character, does nothing
/// If EOF is found, returns an error
fn advance_until_non_whitespace(chars: &Vec<char>, pos: &mut FilePosition) -> Result<(), ParseError> {
    if pos.absolute_position >= chars.len() {
        return Err(ParseError { 
            message: String::from("Unexpected end of file. Maybe you forgot to close a tag."), 
            position: pos.clone(), 
            length: 1 
        });
    }

    while chars[pos.absolute_position].is_whitespace() {
        advance_position(pos, chars)?;
    }

    return Ok(());
}


/// Reads a word and moves the cursor (case insensitive, return lowered chars!)
fn read_word(chars: &Vec<char>, pos: &mut FilePosition) -> Result<String, ParseError> {
   advance_until_non_whitespace(chars, pos)?;
    let mut res = Vec::with_capacity(10);

    while (*pos).absolute_position < chars.len() && 
        (chars[(*pos).absolute_position].is_alphanumeric() || WORD_CHARS.contains(chars[(*pos).absolute_position])) {
        res.push(chars[(*pos).absolute_position]);
        advance_position(pos, chars)?;
    }

    return Ok(res.into_iter().collect::<String>().to_lowercase());
}


/// Reads everything until a quote. The cursor is left after the quote (case insensitive, return lowered chars!)
fn read_until_quote(chars: &Vec<char>, pos: &mut FilePosition) -> Result<String, ParseError> {
    let mut res = Vec::with_capacity(15);

    let start_pos = pos.to_owned();

    while chars[pos.absolute_position] != '"' {
        res.push(chars[(*pos).absolute_position]);

        match advance_position_with_comments(pos, chars) {
            Ok(()) => {},
            Err(_) => return Err(ParseError {
                message: String::from("Unmatched \"."),
                position: start_pos,
                length: 1,
            })
        }
    }
    
    advance_position(pos, chars)?;

    return Ok(res.into_iter().collect::<String>().to_lowercase());
}


/// Reads a word without moving the cursor
fn lookahead_word(chars: &Vec<char>, pos: &mut FilePosition) -> Result<String, ParseError> {
    return read_word(chars, &mut pos.clone());
}


/// Advances a position, updating everything in the struct. Ignores "//" and "/**/" comments.
pub fn advance_position(pos: &mut FilePosition, file: &Vec<char>) -> Result<(), ParseError> {
    let mut in_double_slash_comment = false;
    let mut in_slash_star_comment = false;

    loop {
        (*pos).absolute_position += 1;
        (*pos).line_character += 1;

        if pos.absolute_position >= file.len() {
            return Err(ParseError { 
                message: String::from("Unexpected end of file. Maybe you forgot to close a tag."), 
                position: pos.clone(), 
                length: 1 
            });
        }
        
        let character_before = file[pos.absolute_position - 1];
        let character = 
            if pos.absolute_position < file.len() { // Makes sure nto reading after end of file
                file[pos.absolute_position]
            } else { '\0' };
        let character_after = 
            if pos.absolute_position < file.len() - 1 {
                file[pos.absolute_position + 1]
            } else { '\0' };
            
        // FIXME: will not count lines for os that use \r
        if character_before == '\n' {
            (*pos).line += 1;
            (*pos).line_character = 0;
        }

        // Keep reading if in comments
        if in_double_slash_comment {
            if character == '\n' {
                in_double_slash_comment = false;
            }
        }
        else if in_slash_star_comment {
            if character_before == '*' && character == '/' {
                in_slash_star_comment = false
            }
        }
        else if character == '/' && character_after == '/' { // Detect "//" comments
            in_double_slash_comment = true;
        }
        else if character == '/' && character_after == '*' { // Detect "/**/" comments
            in_slash_star_comment = true;
        }
        else { break };
    }

    return Ok(());
}


/// Same as `advance_position`, but reads comments.
pub fn advance_position_with_comments(pos: &mut FilePosition, file: &Vec<char>) -> Result<(), ParseError> {
    (*pos).absolute_position += 1;
    (*pos).line_character += 1;

    if pos.absolute_position >= file.len() {
        return Err(ParseError { 
            message: String::from("Unexpected end of file. Maybe you forgot to close a tag."), 
            position: pos.clone(), 
            length: 1 
        });
    }
    
    let character_before = file[pos.absolute_position - 1];
    
    // FIXME: will not count lines for os that use \r
    if character_before == '\n' {
        (*pos).line += 1;
        (*pos).line_character = 0;
    }
        
    return Ok(());
}


fn get_positions_difference(a: &FilePosition, b: &FilePosition) -> usize {
    if a.file_path != b.file_path {
        panic!("Tried to compare positions located in different files");
    }

    return a.absolute_position - b.absolute_position;
}


/// basically call advance_position `count` times
pub fn advance_position_many(pos: &mut FilePosition, file: &Vec<char>, count: usize) -> Result<(), ParseError> {
    for _ in 0..count {
        advance_position(pos, file)?;
    }

    return Ok(());
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


// Creates a simple span with text in it. Also specifies a file position
fn get_tag_from_raw_text(text: &str, pos: &FilePosition) -> Node {
    let chars = text.chars();
    let mut content = Vec::with_capacity(chars.clone().count());

    for char in chars {
        content.push(NodeContent::Character((char, pos.clone())));
    }
    
    let res = Node {
        name: String::from("span"),
        attributes: vec![],
        children: vec![],
        content,
        auto_closing: false,
        declaration_symbol: TagSymbol::NOTHING,
        start_position: pos.clone(),
        start_inner_position: pos.clone(),
        source_length: 1,
    };

    return res;
}

