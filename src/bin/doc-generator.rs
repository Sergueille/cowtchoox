use std::fs;


fn main() {
    let math_head = regex::Regex::new(
        r"(?mU)// (?P<desc>.*) ?(Alias (?P<alias>.*))?(Infix alias (?P<inf_alias>.*))?\n(?P<args>(//(.*)\n)*)<\?(?P<name>[^:=<>]+)(?P<attr> .*)?>"
    ).unwrap();

    let tag_head = regex::Regex::new(
        r"(?mU)// (?P<desc>.*) ?(Alias (?P<alias>.*))?(Infix alias (?P<inf_alias>.*))?\n(?P<args>(//(.*)\n)*)<!(?P<name>[^:=<>]+)(?P<attr> .*)?>"
    ).unwrap();

    let file_desc = regex::Regex::new(
        r"// Description: (?P<desc>.*)"
    ).unwrap();

    let file_title = regex::Regex::new(
        r"// Title: (?P<title>.*)"
    ).unwrap();

    let args: Vec<String> = std::env::args().collect();
    let in_file: String;
    let out_file: String;

    if args.len() == 3 {
        in_file = args[1].clone();
        out_file = args[2].clone();
    }
    else {
        println!("Expected 2 argument.");
        show_help();
        return;
    }

    let bytes = fs::read(in_file).unwrap();
    let text = String::from_utf8(bytes).unwrap().replace('\r', "");

    let mut res = String::from("
<document>
    <head>
    <title></title>
    <css>docs.css</css>
    <footer relative-to=\"default-dir\">default/footer.cowx</footer>
    </head>
    <body>
    ");

    match file_title.captures(&text) {
        Some(capt) => {
            res.push_str(&format!("<h1>{}</h1>", capt.name("title").unwrap().as_str()));
        },
        None => {},
    }

    match file_desc.captures(&text) {
        Some(capt) => {
            res.push_str(capt.name("desc").unwrap().as_str());
        },
        None => {},
    }

    res.push_str("<h2>Tags</h2>");

    for capture in tag_head.captures_iter(&text) {
        res.push_str(&parse_tag(capture, false));
    }

    res.push_str("<h2>Math operators</h2>");

    for capture in math_head.captures_iter(&text) {
        res.push_str(&parse_tag(capture, true));
    }

    res.push_str("</body></document>");

    fs::write(out_file, res).unwrap();

    println!("Done!");
}

fn parse_tag(capture: regex::Captures, math: bool) -> String {
    let arg_match = regex::Regex::new(
        r"// (?P<arg>.*): (?P<arg_desc>.*)"
    ).unwrap();

    let inline_arg_match = regex::Regex::new(
        r":(?P<arg>\w*)"
    ).unwrap();

    let desc = capture.name("desc").expect("No description.").as_str();
    let alias = capture.name("alias");
    let inf_alias = capture.name("inf_alias");
    let args = capture.name("args").expect("Uuh?").as_str();
    let name = capture.name("name").expect("No name.").as_str();
    let attr = capture.name("attr");

    let alias_text = match (alias, inf_alias) {
        (None, None) => String::new(),
        (None, Some(inf_alias)) => format!("<div class=\"alias\">Infix alias `{}`</div>", inf_alias.as_str()),
        (Some(alias), _) => format!("<div class=\"alias\">Alias `{}`</div>", alias.as_str()),
    };

    let mut args_text = String::new();
    let mut inline_args_text = String::new();
    let mut autoclosing = true;

    for arg_capture in arg_match.captures_iter(args) {
        let arg_name = arg_capture.name("arg").expect("No arg name.").as_str();
        let arg_desc = arg_capture.name("arg_desc").expect("No arg description.").as_str();

        args_text.push_str(&format!("`{}`: {} <br/>\n", arg_name, arg_desc));
    }

    match attr {
        Some(attr) => {
            for arg_capture in inline_arg_match.captures_iter(attr.as_str()) {
                let arg_name = arg_capture.name("arg").unwrap().as_str();

                if math {
                    inline_args_text.push_str(&format!("{{{}}}", arg_name));
                }
                else {
                    if arg_name == "inner" {
                        autoclosing = false;
                        continue;
                    }

                    inline_args_text.push_str(&format!(":{}=\"\"", arg_name));
                }
            }
        },
        None => {},
    };

    if !math {
        if autoclosing {
            return format!(
                "<h3>`<{}>` {}</h3>\n``<!{} {}/>``\n{} <br/>\n{}\n\n", 
                name, alias_text, name, inline_args_text, desc, args_text
            );   
        }
        else {
            return format!(
                "<h3>`<{}>` {}</h3>\n``<!{} {}> </{}>``\n{} <br/>\n{}\n\n", 
                name, alias_text, name, inline_args_text, name, desc, args_text
            );   
        }
 
    }
    else  {
        return format!(
            "<h3>`{}` {}</h3>\n``?{}{}``\n<mathnode class=\"center\">?{}{}</mathnode>\n{} <br/>\n{}\n\n", 
            name, alias_text, name, inline_args_text, name, inline_args_text, desc, args_text
        );    
    }

} 

fn show_help() {
    println!("");
    println!("Help:");
    println!("Automatically crates a documentation file from comments. See default/default.cowx to see how to use the comments.");
    println!("Usage: cargo run --bin doc-generator -- [COWX FILE] [OUTPUT FILE]");
}

