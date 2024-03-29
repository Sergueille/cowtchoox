use std::{path::PathBuf, rc::Rc};

// Some useful structs and functions


/// Indicates a position in a file
/// All fields start at 0. Even lines.
#[derive(Clone, Debug)]
pub struct FilePosition {
    pub file_path: Rc<PathBuf>,
    pub absolute_position: usize,
    pub line: usize,
    pub line_character: usize
}

