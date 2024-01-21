
use crate::parser::Node;
use crate::doc_options;

// Transform the struct back to raw HTML


// Get the entire text of the document, ready for being displayed
pub fn get_file_text(document: &Node) -> Result<String, ()> {
    let mut res = String::new();

    let head = match try_get_document_head(document) {
        Ok(head) => head,
        Err(()) => {
            // TODO: warn
            return Err(());
        }
    };
    let options = doc_options::get_options_form_head(head);

    res.push_str("<document>"); // Quirks forever!

    res.push_str(&white_head(&options));

    // TODO: write the body

    res.push_str("</document>");

    return Ok(res);
}


pub fn white_head(options: &doc_options::DocOptions) -> String {
    let mut res = String::with_capacity(200);
    res.push_str("<head>");

    res.push_str(format!("<title>{}</title>", options.title).as_str());

    res.push_str("</head>");
    return res;
}


/// Looks for the head of a document, returns Err if not found
pub fn try_get_document_head<'a>(document: &'a Node) -> Result<&'a Node, ()> {
    for child in &document.children {
        if child.name == "head" {
            return Ok(child);
        }
    }

    return Err(());
}

