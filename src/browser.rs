
use std::path::PathBuf; 
use std::fs;
use std::time::Duration;
use crate::Args;
use crate::doc_options::DocOptions;

// Sends the file to the browser!

pub fn render_to_pdf(path: PathBuf, args: &Args, options: &DocOptions) {
    // create the browser
    let browser = headless_chrome::Browser::new(
        headless_chrome::LaunchOptions { 
            headless: !args.headful,
            ..Default::default()
        }).expect("Could'n find chromium!");

    let tab = browser.new_tab().expect("Can't crate new tab");

    // Navigate to the page
    tab.navigate_to(&format!("file:///{}", &path.clone().into_os_string().into_string().expect("")))
        .expect("Failed to navigate to document");

    std::thread::sleep(Duration::from_secs(1)); // FIXME: needs to wait for JS to finish executing, not just waiting 1 sec

    // Export tp pdf
    let pdf = tab.print_to_pdf(Some(headless_chrome::types::PrintToPdfOptions {
        display_header_footer: Some(false),
        // FIXME: it seems that the page element overflows 1px on the pdf page because of precision issues 
        paper_width: Some(options.format.width as f64 / 30.7),
        paper_height: Some(options.format.height as f64 / 30.7), // Convert from mm to px (https://developer.mozilla.org/en-US/docs/Web/CSS/length#absolute_length_units)
        print_background: Some(true),
        margin_bottom: Some(0.0),
        margin_top: Some(0.0),
        margin_left: Some(0.0),
        margin_right: Some(0.0),
        scale: Some(1.0), // No idea of what it is...
        ..Default::default()
    })).unwrap();

    // FIXME: someone who knows rust please help
    let mut ancestors = path.ancestors();
    ancestors.next();
    let mut pdf_path = ancestors.next().unwrap().to_path_buf();
    pdf_path.push("out.pdf");
    println!("{:?}", pdf_path);
    fs::write(pdf_path, pdf).unwrap();

    if args.keep_alive {
        println!("Keeping the browser alive forever, stop it manually");
        loop {}
    }

}


