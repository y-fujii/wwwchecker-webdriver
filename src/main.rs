use std::*;
mod webdriver;

fn main() -> Result<(), Box<dyn error::Error>> {
    let driver = webdriver::WebDriver::new(
        &mut process::Command::new("geckodriver"),
        //&mut process::Command::new("chromedriver"),
        serde_json::json!({
            "alwaysMatch": {
                "moz:firefoxOptions": {
                    "binary": "firefox-developer-edition",
                    "args": ["-headless"],
                    "prefs": { "layout.css.devPixelsPerPx": "2" },
                },
                "goog:chromeOptions": {
                    // XXX: scale factor.
                    "args": ["-headless", "-hide-scrollbars"],
                },
            },
        }),
        4444,
    )?;
    driver.set_window_size(1024, 1024)?;
    driver.set_url("https://mimosa-pudica.net/")?;
    let _text: String = driver.execute("return document.body.textContent")?;
    //thread::sleep(time::Duration::from_millis(500));
    let body = driver.element("html")?;
    let data = driver.element_screenshot(&body)?;

    fs::write("test0.png", &data).unwrap();

    Ok(())
}
