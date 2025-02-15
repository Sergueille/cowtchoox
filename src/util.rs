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


pub fn get_browser_path_string(path: PathBuf, with_file_prefix: bool) -> String {
    let mut path_str = path.display().to_string();
    path_str = path_str.replace('\\', "/").replace("//?/", ""); // HACK: sometimes "//?/" appears, don't know why

    let file_prefix = if with_file_prefix { "" } else { "" };
    return format!("{}{}", file_prefix, path_str);
}
