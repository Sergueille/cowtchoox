
use crate::parser::Node;
use crate::doc_options;

// Transform the struct back to raw HTML


// Get the entire text of the document, ready for being displayed
pub fn get_file_text(document: &Node) -> Result<String, ()> {
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

    return Ok(res);
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

        for content in &node.content {
            match content {
                crate::parser::NodeContent::Character(c) => res.push(*c),
                crate::parser::NodeContent::Child(id) => res.push_str(&get_node_html(&node.children[*id])),
            }
        }

        res.push_str(&format!("</{}>", node.name))
    }

    return res;
}

