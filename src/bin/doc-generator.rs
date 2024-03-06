use std::fs;


fn main() {
    let head = regex::Regex::new(
        r"(?mU)// (?P<desc>.*) ?(Alias (?P<alias>.*))?(Infix alias (?P<inf_alias>.*))?\n(?P<args>(//(.*)\n)*)<\?(?P<name>[^:=<>]+)(?P<attr> .*)?>"
    ).unwrap();

    let arg_match = regex::Regex::new(
        r"// (?P<arg>.*): (?P<arg_desc>.*)"
    ).unwrap();

    let inline_arg_match = regex::Regex::new(
        r":(?P<arg>\w*)"
    ).unwrap();

    let bytes = fs::read("default\\default.cowx").unwrap();
    let text = String::from_utf8(bytes).unwrap().replace('\r', "");

    let mut res = String::new();

    for capture in head.captures_iter(&text) {
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

        for arg_capture in arg_match.captures_iter(args) {
            let arg_name = arg_capture.name("arg").expect("No arg name.").as_str();
            let arg_desc = arg_capture.name("arg_desc").expect("No arg description.").as_str();

            args_text.push_str(&format!("`{}`: {} <br/>\n", arg_name, arg_desc));
        }

        match attr {
            Some(attr) => {
                for arg_capture in inline_arg_match.captures_iter(attr.as_str()) {
                    inline_args_text.push_str(&format!("{{{}}}", arg_capture.name("arg").unwrap().as_str()));
                }
            },
            None => {},
        };

        res.push_str(&format!(
            "<h3>`{}` {}</h3>\n``?{}{}``\n<mathnode class=\"center\">?{}{}</mathnode>\n{} <br/>\n{}\n\n", 
            name, alias_text, name, inline_args_text, name, inline_args_text, desc, args_text
        ));
    }

    println!("Output: \n\n{}", res);
}
