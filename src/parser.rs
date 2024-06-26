
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


// Allowed autoclosing tags
const AUTOCLOSING_TAGS: [&str; 17] = [
    "area", "base", "br", "col", "command", "embed", "hr", "img", "input", "keygen", "link", "meta", "param", "source", "track", "wbr", "path"
];


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
    pub attributes: Vec<TagAttribute>,
    pub children: Vec<Node>,
    pub content: Vec<NodeContent>,
    pub auto_closing: bool,
    pub is_math: bool, // Is this tag a math tag? (If false, can still be a math tag if inside a math tag)
    pub declaration_symbol: TagSymbol,
    pub start_position: FilePosition, // Where it is located in the source file
    pub start_inner_position: FilePosition, // Where the inner content is located in the source file
    pub source_length: usize, // How long it is in th source file
}


/// Represent a tag's attribute
#[derive(Debug, Clone)]
pub struct TagAttribute {
    pub name: String,
    pub value: Option<String>,
    pub position: Option<FilePosition>, // Is None if not defined by the suer but automatically added by cowtchoox 
    pub value_position: Option<FilePosition>, // Same, but is none if no value
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
    Normal, Math, BigMath, Code, BigCode, InsideAttributeValue
}


bitflags::bitflags! {
    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    pub struct TagSymbol: u8 {
        const NOTHING = 0x1;
        const QUESTION_MARK = 0x2;
        const EXCLAMATION_MARK = 0x4;
        const COLON = 0x8;
        const PERCENTAGE = 0x10;
    }
}


#[derive(Clone, Debug)]
struct SplitPosition {
    pub content_pos: usize,
    pub file_pos: FilePosition,
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
    let res = parse_tag(chars, &mut get_start_of_file_position(file_path.clone()), TagSymbol::NOTHING, false, context)?;

    // Make sure it contains no colon tag
    custom::check_colon_tags(&res, &Vec::new())?;

    return Ok(res);
}


/// Parses a part of the raw file. The beginning must be at the start of a node.
/// Does not parse math.
/// 
/// # Arguments
/// * `file`: the raw contents of the file
/// * `pos`: the index of the character where the function should start parsing
/// * `expect_symbol`: the expected thing after the first `<`. For nothing use `TagSymbol::Nothing` 
/// 
/// # Returns
/// * the parsed node
/// 
pub fn parse_tag(chars: &Vec<char>, mut pos: &mut FilePosition, expect_symbol: TagSymbol, math: bool, context: &Context) -> Result<Node, ParseError> {
    expect(chars, &mut pos, '<')?;

    let start_pos = pos.clone();

    let next_char = read_next_char(chars, pos)?;
    let used_symbol = match next_char {
        '?' => TagSymbol::QUESTION_MARK,
        '!' => TagSymbol::EXCLAMATION_MARK,
        ':' => TagSymbol::COLON,
        '%' => TagSymbol::PERCENTAGE,
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
                message: format!("Unexpected \"{}\"", next_char), // TODO: say what it expected
                position: start_pos,
                length: 1,
            });
        }
    }

    let mut attributes = Vec::with_capacity(10);

    // Read tag name
    let tag_name = read_word(chars, &mut pos)?;

    // Read the attributes
    loop {
        let attr_pos = pos.clone();

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
                
                let val_pos;

                let value;
                if use_quotes {
                    advance_position_with_comments(pos, chars)?;
                    val_pos = pos.clone();
                    value = read_until_quote(chars, pos)?;
                }
                else {
                    val_pos = pos.clone();
                    value = read_word(chars, pos)?;
                }

                attributes.push(TagAttribute {
                    name: attr,
                    value: Some(value),
                    position: Some(attr_pos),
                    value_position: Some(val_pos),
                });
            },
            Err(_) => { // No value
                attributes.push(TagAttribute {
                    name: attr,
                    value: None,
                    position: Some(attr_pos),
                    value_position: None,
                });
            },
        }
    }

    // Is auto-closing tag?
    let got_autoclosing_slash = expect(chars, pos, '/');

    // Are we closing the tag correctly?
    expect(chars, pos, '>')?;

    let inner_start_pos = pos.clone();

    let mut res;

    let is_really_math = math || used_symbol == TagSymbol::QUESTION_MARK || tag_name == "mathnode";

    match got_autoclosing_slash {
        Ok(()) => { // Auto-closing

            // Check if it is allowed
            if used_symbol != TagSymbol::EXCLAMATION_MARK 
            && used_symbol != TagSymbol::COLON
            && !AUTOCLOSING_TAGS.contains(&tag_name.as_str()) {
                return Err(
                    ParseError { 
                        message: format!(
                            "The tag \"{}\" should not be auto-closing. Consider using \"<{}></{}>\". The only allowed tags are custom tags, or br, img, link, and few others.", 
                            tag_name, tag_name, tag_name
                        ), 
                        position: start_pos, 
                        length: tag_name.len() + 1
                    }
                );
            }

            // Return the node directly
            let length = get_positions_difference(pos, &start_pos);
            res = Node {
                name: tag_name,
                attributes,
                children: vec![],
                content: vec![],
                auto_closing: true,
                is_math: is_really_math,
                declaration_symbol: used_symbol,
                start_position: start_pos,
                start_inner_position: inner_start_pos,
                source_length: length,
            };
        },
        Err(_) => { // Not auto-closing

            // Throw error if colon used
            if used_symbol == TagSymbol::COLON {
                return Err(
                    ParseError { 
                        message: String::from(
                            "A tag that starts with a colon should be autoclosing. If it's not meant to be an argument for a custom tag, the colon shouldn't be here"
                        ), 
                        position: start_pos, 
                        length: tag_name.len() + 1
                    }
                );
            }

            res = Node {
                name: tag_name,
                attributes,
                children: vec![],
                content: vec![],
                auto_closing: false,
                is_math: is_really_math,
                declaration_symbol: used_symbol,
                start_position: start_pos,
                start_inner_position: inner_start_pos,
                source_length: 0,
            };

            parse_inner_tag(chars, &mut res, pos, if is_really_math { ParserState::Math } else { ParserState::Normal }, true, context)?;
            expect(chars, &mut pos, '<')?;
            expect(chars, &mut pos, '/')?;
            
            // Parse the node's contents
            // Got out of the contents, now cursor is in closing tag
            let closing_tag_name = lookahead_word(chars, pos)?;
            if closing_tag_name != res.name {
                // Show a hint in math
                let math_hint = if is_really_math { 
                    get_math_close_tag_hint(&closing_tag_name)
                } else { 
                    String::new()
                };

                return Err(ParseError { 
                    message: format!("Unmatched tag. Expected to close tag \"{}\", but found tag \"{}\".{}", res.name, closing_tag_name, math_hint), 
                    position: pos.clone(), 
                    length: closing_tag_name.len() 
                });
            }
            read_word(chars, pos)?; // Advance cursor to after the tag name 

            // Check for the very last character...
            expect_allow_eof(chars, pos, '>')?;

            res.source_length = get_positions_difference(pos, &res.start_position);
        },
    }

    // Yay, the user gave a completely valid node!
    
    return Ok(res);
}


/// Parses text inside a tag. Helper for `parse_tag`
fn parse_inner_tag<'a>(chars: &Vec<char>, node: &'a mut Node, pos: &mut FilePosition, state: ParserState, allow_closing_tag: bool, context: &Context) -> Result<(), ParseError> {
    let mut children: Vec<Node> = Vec::with_capacity(10);
    let mut content: Vec<NodeContent> = Vec::with_capacity(100);
    
    let mut backslashed_character = false; // Should the next character be ignored because of a backslash

    let mut simple_splits: Vec<SplitPosition> = Vec::new();
    let mut double_splits: Vec<SplitPosition> = Vec::new();

    let start_pos = pos.clone();

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

            let mut lookahead_pos = pos.clone();
            advance_position(&mut lookahead_pos, chars)?;
            let next_char = read_next_char(chars,&mut lookahead_pos)?;

            match next_char {
                '/' => { // Reached closing tag
                    if !allow_closing_tag {
                        let close_tag_name = read_word(chars, &mut lookahead_pos)?;

                        // Show an additional hint in math
                        let math_hint = if state == ParserState::Math || state == ParserState::BigMath {
                            get_math_close_tag_hint(&close_tag_name)
                        }
                        else {
                            String::new()
                        };

                        return Err(ParseError {
                            message: format!("Unexpected closing tag \"</{}>\".{}", close_tag_name, math_hint),
                            position: pos.clone(),
                            length: 3 + close_tag_name.chars().count(),
                        });
                    }

                    break;
                },
                _ => { 
                    let in_math = state == ParserState::Math || state == ParserState::BigMath;
                    let real_math_tag = next_char == '%' || next_char == '!'  || next_char == ':';
                    if (in_math && real_math_tag) || !in_math {
                         // Opening tag
                        let mut res_pos = pos.clone();
                        let allowed_symbols = if in_math { TagSymbol::PERCENTAGE | TagSymbol::EXCLAMATION_MARK | TagSymbol::COLON }
                                              else       { TagSymbol::NOTHING    | TagSymbol::EXCLAMATION_MARK | TagSymbol::COLON };

                        let result = parse_tag(
                            chars, 
                            &mut res_pos, 
                            allowed_symbols, 
                            in_math,
                            context
                        );

                        match result {
                            Ok(child) => {
                                children.push(child);
                                content.push(NodeContent::Child(children.len() - 1));
                                *pos = res_pos;
                            }
                            Err(e) => {
                                return Err(e);
                            },
                        }
                    }
                    else {
                        // Regular math text
                        content.push(NodeContent::Character((next, pos.clone())));
                        advance_position(pos, chars)?;
                    }
                }
            }
        }
        else if next == '&' {
            let ampersand_after = chars.len() > pos.absolute_position + 1 && chars[pos.absolute_position + 1] == '&';
            
            if ampersand_after {
                double_splits.push(SplitPosition { content_pos: content.len(), file_pos: pos.clone()});
                advance_position(pos, chars)?;
            }
            else {
                simple_splits.push(SplitPosition { content_pos: content.len(), file_pos: pos.clone()});
            }

            advance_position(pos, chars)?;
        }
        else if next == '$' {
            let pos_before_dollar = pos.clone();
            advance_position(pos, chars)?;

            let double = chars[pos.absolute_position] == '$';
            if double {
                advance_position(pos, chars)?;
            }

            match state {
                ParserState::BigMath => {
                    if !double {
                        return Err(ParseError {
                            message: String::from("Found one dollar, but expected two (\"$$\")."),
                            position: pos_before_dollar,
                            length: 1,
                        });
                    }

                    break;
                },
                ParserState::Math => {
                    if double {
                        return Err(ParseError {
                            message: String::from("Found two dollars, but expected one (\"$\")."),
                            position: pos_before_dollar,
                            length: 1,
                        });
                    }

                    break;
                },
                _ => {
                    let mut start_inner_position = pos.clone();
                    advance_position(&mut start_inner_position, chars)?;
    
                    let attributes = if double {
                        vec![
                            TagAttribute { 
                                name: String::from("class"), 
                                value: Some(String::from("center")), 
                                position: None, 
                                value_position: None,
                            }
                        ]
                    }
                    else {
                        vec![
                            TagAttribute { 
                                name: String::from("nonbreaking"), 
                                value: None, 
                                position: None, 
                                value_position: None,
                            }
                        ]
                    };
    
                    let mut math_tag = Node {
                        name: String::from("mathnode"),
                        attributes,
                        children: vec![],
                        content: vec![],
                        auto_closing: false,
                        is_math: true,
                        declaration_symbol: TagSymbol::NOTHING,
                        start_position: pos.clone(),
                        start_inner_position,
                        source_length: 0
                    };
    
                    let math_type = if double { ParserState::BigMath } else { ParserState::Math }; 
    
                    parse_inner_tag(chars, &mut math_tag, pos, math_type, false, context)?;
    
                    math_tag.source_length = get_positions_difference(&pos, &math_tag.start_position);
    
                    children.push(math_tag);
                    content.push(NodeContent::Child(children.len() - 1));
                }
            }
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
                is_math: false,
                declaration_symbol: TagSymbol::NOTHING,
                start_position: pos.clone(),
                start_inner_position,
                source_length: 0
            };
            
            if double {
                advance_position_with_comments(pos, chars)?;
            } 

            parse_inner_tag(chars, &mut math_tag, pos, if double { ParserState::BigCode } else { ParserState::Code }, false, context)?;

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

        // Is it the end of the file?
        if pos.absolute_position >= chars.len() - 1 {
            if state == ParserState::InsideAttributeValue { // It's normal, we're inside an attribute!
                break;
            }
            else {
                // Uuh, continue the loop, the next one who wants to get a character will throw en error
            }
        }
    }

    if double_splits.len() > 0 || simple_splits.len() > 0 {
        double_splits.insert(0, SplitPosition { content_pos: 0, file_pos: start_pos });
        double_splits.push(SplitPosition { content_pos: content.len(), file_pos: pos.clone() });

        // Split &&
        let (actual_content, mut actual_children) = split_ampersands(content, children, &double_splits, "double-amp-split");

        let mut simple_splits_id = 0;

        for (i, child) in actual_children.iter_mut().enumerate() {

            // Check for simple splits that are inside the child
            while simple_splits_id < simple_splits.len() && simple_splits[simple_splits_id].content_pos < double_splits[i].content_pos {
                simple_splits_id += 1;
            }

            let mut child_splits = Vec::new();
            while simple_splits_id < simple_splits.len() && simple_splits[simple_splits_id].content_pos < double_splits[i + 1].content_pos {
                child_splits.push(SplitPosition {
                    content_pos: simple_splits[simple_splits_id].content_pos - double_splits[i].content_pos,
                    file_pos: double_splits[i].file_pos.clone(),
                });
                simple_splits_id += 1;
            }

            // If there are & inside, split again
            if child_splits.len() > 0 {
                child_splits.insert(0, SplitPosition { content_pos: 0, file_pos: double_splits[i].file_pos.clone() });
                child_splits.push(SplitPosition { 
                    content_pos: double_splits[i + 1].content_pos - double_splits[i].content_pos, 
                    file_pos: double_splits[i + 1].file_pos.clone() 
                });

                let moved_content = std::mem::replace(&mut child.content, vec![]);
                let moved_children = std::mem::replace(&mut child.children, vec![]);
                
                let (actual_child_content, actual_child_children) = split_ampersands(moved_content, moved_children, &child_splits, "amp-split");
                
                child.children = actual_child_children;
                child.content = actual_child_content;
            }
        }

        node.children = actual_children;
        node.content = actual_content;
    } 
    else {
        node.children = children;
        node.content = content;
    }

    return Ok(());
}


fn split_ampersands(content: Vec<NodeContent>, children: Vec<Node>, split_positions: &Vec<SplitPosition>, split_tag_name: &str) -> (Vec<NodeContent>, Vec<Node>) {
    let mut actual_content = Vec::with_capacity(split_positions.len() - 1);
    let mut actual_children = Vec::with_capacity(split_positions.len() - 1);

    let mut content_option: Vec<_> = content.into_iter().map(|el| Some(el)).collect();
    let mut children_option: Vec<_> = children.into_iter().map(|el| Some(el)).collect();

    for i in 0..(split_positions.len() - 1) {
        let pos = &split_positions[i].file_pos;

        let mut splitted = Node {
            name: String::from(split_tag_name),
            attributes: vec![],
            children: vec![],
            content: vec![],
            auto_closing: false,
            is_math: false,
            declaration_symbol: TagSymbol::NOTHING,
            start_position: pos.clone(),
            start_inner_position: pos.clone(),
            source_length: split_positions[i + 1].file_pos.absolute_position - split_positions[i].file_pos.absolute_position,
        };

        for content_id in (split_positions[i].content_pos)..(split_positions[i + 1].content_pos) {
            let content_item = std::mem::replace(&mut content_option[content_id], None).unwrap();

            match content_item {
                NodeContent::Child(c) => {
                    splitted.content.push(NodeContent::Child(splitted.children.len()));

                    let child = std::mem::replace(&mut children_option[c], None).unwrap();
                    splitted.children.push(child);
                },
                _ => {
                    splitted.content.push(content_item);
                }
            }
        }

        actual_content.push(NodeContent::Child(actual_children.len()));
        actual_children.push(splitted);
    }

    return (actual_content, actual_children);
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
pub fn get_attribute_value<'a>(node: &'a Node, attrib_name: &str) -> Result<Option<&'a str>, ()> {
    for attr in &node.attributes {
        if attrib_name == attr.name {
            return Ok(attr.value.as_deref());
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


/// Same as `expect`, but does not throw if EOF is just after the character
fn expect_allow_eof(chars: &Vec<char>, pos: &mut FilePosition, char: char) -> Result<(), ParseError> {
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

    // Ignore the EOF error
    let _ = advance_position(pos, chars);

    return Ok(());
}


/// Advances the cursor until non-whitespace is found (or end of file) (does nothing if already on non-whitespace). Returns an error if cursor on EOF 
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

    return Ok(res.into_iter().collect::<String>());
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

    return Ok(res.into_iter().collect::<String>());
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


// Creates a tag from the text file and a file position (will parse inner tags)
pub fn get_tag_from_raw_text(text: &str, is_math: bool, pos: &FilePosition, context: &Context) -> Result<Node, ParseError> {
    let chars = text.chars().collect();
    
    let mut res = Node {
        name: String::from("span"),
        attributes: vec![],
        children: vec![],
        content: vec![],
        auto_closing: false,
        is_math,
        declaration_symbol: TagSymbol::NOTHING,
        start_position: pos.clone(),
        start_inner_position: pos.clone(),
        source_length: 1,
    };

    let mut fake_pos = pos.clone();
    fake_pos.absolute_position -= pos.absolute_position; // Set a false position, so that 0 is the beginning of the string

    match parse_inner_tag(&chars, &mut res, &mut fake_pos, ParserState::InsideAttributeValue, false, context) {
        Ok(()) => Ok(res),
        Err(mut err) => {
            err.position.absolute_position += pos.absolute_position; // Put the right position again
            return Err(err);
        },
    }
}

fn get_math_close_tag_hint(tag_name: &str) -> String {
    return format!(
        " Hint: It happens in a math environnement, so you maybe you forgot the percentage sign on the corresponding opening tag, as it is required in math: \"<%{}></{}>\"",
        tag_name,
        tag_name
    )
}

