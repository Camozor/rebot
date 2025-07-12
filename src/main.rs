use std::time::Duration;

use headless_chrome::{protocol::cdp::Page, Browser};

fn main() {
    let browser = Browser::default().unwrap();
    let tab = browser.new_tab().unwrap();

    // tab.enable_request_interception

    tab.navigate_to("https://u.gg/rematch/profile/steam/La%20m%C3%A9sange%20du%20Val%20d'Oise/76561198355389674").unwrap();
    // https://u.gg/rematch/profile/steam/La%20m%C3%A9sange%20du%20Val%20d'Oise/76561198355389674
    tab.wait_until_navigated().unwrap();

    std::thread::sleep(Duration::from_secs(10));

    let jpeg_data = tab
        .capture_screenshot(Page::CaptureScreenshotFormatOption::Jpeg, None, None, true)
        .unwrap();
    std::fs::write("screenshot.jpeg", jpeg_data).unwrap();
}
