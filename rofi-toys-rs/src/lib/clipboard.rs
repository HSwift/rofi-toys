use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use arboard::Clipboard;
use once_cell::sync::Lazy;

static CLIPBOARD: Lazy<Mutex<Clipboard>> =
    Lazy::new(|| Mutex::new(Clipboard::new().expect("create clipboard object error")));

pub fn get_clipboard_text() -> String {
    let mut clipboard = CLIPBOARD.lock().unwrap();
    clipboard
        .get_text()
        .expect("get text from clipboard failed")
}

pub fn set_clipboard_text(text: &str) {
    let mut clipboard = CLIPBOARD.lock().unwrap();
    clipboard.set_text(text).expect("set clipboard text failed");

    if cfg!(target_os = "linux") {
        thread::sleep(Duration::from_millis(100));
    }
}
