
use crate::parser::{Node, NodeContent};
use crate::doc_options::{self, DocOptions};

// Transform the struct back to raw HTML
// NOTE: all text will be wrapped in <text> tags


// Get the entire text of the document, ready for being displayed
pub fn get_file_text(document: &Node) -> Result<(String, DocOptions), ()> {
    let mut res = String::new();

    let head = match try_get_children_with_name(document, "head") {
        Ok(head) => head,
        Err(()) => {
            // TODO: warn
            return Err(());
        }
    };
    let options = doc_options::get_options_form_head(head);

    res.push_str("<html>"); // Quirks is better!

    res.push_str(&white_head(&options));

    res.push_str(&get_node_html(&try_get_children_with_name(document, "body").expect("The document has no body")));

    res.push_str("</html>");

    return Ok((res, options));
}


pub fn white_head(options: &doc_options::DocOptions) -> String {
    let mut res = String::with_capacity(200);
    res.push_str("<head>");

    // Document title
    res.push_str(format!("<title>{}</title>", options.title).as_str());

    // FIXME: should be like ~"path_to_exe/" when built, and ~"" when running with cargo
    //        but too lazy to do that
    let default_resources_path = "";

    // Link JS script, so that it executes when the page loads
    res.push_str(&format!("<script defer=\"defer\" src=\"{}JS/main.js\"></script>", default_resources_path));

    // Link default CSS
    res.push_str(&format!("<link rel=\"stylesheet\" href=\"{}css/default.css\"/>", default_resources_path));

    // Page size
    res.push_str(&format!("<meta name=\"pagewidth\" content=\"{}\"/>", options.format.width));
    res.push_str(&format!("<meta name=\"pageheight\" content=\"{}\"/>", options.format.height));

    res.push_str("</head>");
    return res;
}


/// Looks for the head of a document, returns Err if not found
pub fn try_get_children_with_name<'a>(document: &'a Node, name: &str) -> Result<&'a Node, ()> {
    for child in &document.children {
        if child.name == name {
            return Ok(child);
        }
    }

    return Err(());
}


/// Generates HTML for a node
/// TEST: the function is not actually implemented properly
///       this is directly reconstructing the tag without processing anything
pub fn get_node_html(node: &Node) -> String {
    let mut res = String::from("<");

    res.push_str(&node.name);
    res.push(' ');
    
    for (attr, val) in &node.attributes {
        res.push_str(&format!("{}=\"{}\" ", &attr, &val));
    }    

    if node.auto_closing {
        res.push_str("/>");
    }
    else {
        res.push('>');

        let mut previous: &NodeContent = &NodeContent::Child(0); // Keep track of the last character
        for content in &node.content {
            match content {
                crate::parser::NodeContent::Character(c) => {
                    match previous {
                        NodeContent::Character(_) => {},
                        NodeContent::Child(_) => {
                            res.push_str("<text>"); // Begin text tag
                        },
                    }

                    res.push(*c)
                },
                crate::parser::NodeContent::Child(id) => {
                    match previous {
                        NodeContent::Character(_) => {
                            res.push_str("</text>"); // End text tag
                        },
                        NodeContent::Child(_) => {},
                    }

                    res.push_str(&get_node_html(&node.children[*id]))
                },
            }

            previous = content;
        }

        res.push_str(&format!("</{}>", node.name))
    }

    return res;
}

