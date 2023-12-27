mod http_rpc;
mod manager;
mod stroage;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use manager::ClipboardManager;
use rofi_toys::{clipboard, file};

#[derive(serde::Serialize, serde::Deserialize)]
struct ClipdConfig {
    max_size: usize,
}

fn generate_default_config() -> ClipdConfig {
    let clipd_config = ClipdConfig { max_size: 32 };
    file::config_save_to_file(&clipd_config, "clipd").unwrap();
    return clipd_config;
}

fn read_config() -> ClipdConfig {
    match file::config_restore_from_file("clipd") {
        Ok(x) => x,
        Err(_) => generate_default_config(),
    }
}

fn main() {
    let app = Application::builder()
        .application_id("com.rofi-toys.clipd")
        .build();

    app.connect_activate(|app| {
        let clipd_config = read_config();
        let clipboard = gtk::Clipboard::get(&gtk::gdk::SELECTION_CLIPBOARD);
        let manager =
            ClipboardManager::new(clipd_config.max_size, clipboard::get_clipd_listen_path());

        clipboard.connect("owner-change", false, move |values| {
            let clipboard = values[0]
                .get::<gtk::Clipboard>()
                .expect("clipboard cast failed");

            manager.clipboard_on_update(clipboard);
            return None;
        });

        // 剪贴板需要一个窗口才能收到消息
        // 所以建一个空的窗口
        let _ = ApplicationWindow::builder()
            .application(app)
            .default_width(0)
            .default_height(0)
            .title("clipd")
            .build();
    });

    app.run();
}
