
use std::path::PathBuf; 
use std::time::Duration;
use std::{fs, thread};
use crate::Args;
use crate::doc_options::DocOptions;
use crate::log;

// Sends the file to the browser!

pub fn render_to_pdf(path: PathBuf, args: &Args, options: &DocOptions) -> Result<(), ()> {
    // create the browser
    log::log("Opening the browser...");
    let browser = log::log_if_err(headless_chrome::Browser::new(
        headless_chrome::LaunchOptions { 
            headless: !args.headful,
            ..Default::default()
        }), "Couldn't find chromium on your computer. In order to create PDFs, a chromium must be installed.")?;

    let tab = log::log_if_err(browser.new_tab(), "Can't crate new tab")?;

    log::log("Splitting pages...");

    // Navigate to the page
    let res = tab.navigate_to(&format!("file:///{}", &path.clone().into_os_string().into_string().expect("")));
    log::log_if_err(res, "Failed to navigate to document (1).")?;
    log::log_if_err(tab.wait_until_navigated(), "Failed to navigate to document (2).")?;

    const SCALE_CONSTANT: f64 = 1.21; // A random constant to make things work
    const MM_TO_PX: f64 = 1.0 / 30.7; // Convert from mm to px (https://developer.mozilla.org/en-US/docs/Web/CSS/length#absolute_length_units)

    // Check for error reporting element: if it's produced, it's finished! If not, wait.
    let errors_object;
    loop {
        let try_errors_object = tab.find_element("#cowtchoox-error-reporter");

        match try_errors_object {
            Ok(obj) => {
                errors_object = obj;
                break;
            },
            Err(_) => {}, // Wait more
        }

        thread::sleep(Duration::from_millis(200));
    }

    log::log("Creating PDF...");

    // Export tp pdf
    let pdf = tab.print_to_pdf(Some(headless_chrome::types::PrintToPdfOptions {
        display_header_footer: Some(false),
        // FIXME: it seems that the page element overflows 1px on the pdf page because of precision issues 
        paper_width: Some(options.format.width as f64 * MM_TO_PX * SCALE_CONSTANT),
        paper_height: Some(options.format.height as f64 * MM_TO_PX * SCALE_CONSTANT), 
        print_background: Some(true),
        margin_bottom: Some(0.0),
        margin_top: Some(0.0),
        margin_left: Some(0.0),
        margin_right: Some(0.0),
        scale: Some(SCALE_CONSTANT), // No idea of what it is...
        ..Default::default()
    })).unwrap();

    let mut pdf_path = path.parent().unwrap().to_path_buf();
    pdf_path.push("out.pdf");
    fs::write(pdf_path, pdf).unwrap();

    // Collect errors, stored in an html element. (Yes, it's ridiculous, but evaluate() isn't working)
    let all_errors = errors_object.get_inner_text().unwrap();
    
    if all_errors.len() > 0 {            
        for message in all_errors.split('\0') {
            log::warning(&format!("The browser is complaining: {}", message));
        }
    }

    // Cleanup

    if args.keep_alive {
        log::log("Keeping the browser alive forever, stop it manually");
        loop {}
    }

    let res = tab.close(true);
    log::log_if_err(res, "Failed to close browser tab.")?;

    // NOTE: some background thread is panicking just before exit, so I added that to hide the error message
    //       it's not so bad because it's the last thing that is done.
    std::panic::set_hook(Box::new(|_info| {
        // Do nothing if got a panic!
    }));

    return Ok(());
}


