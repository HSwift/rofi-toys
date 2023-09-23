mod manager;
mod stroage;
mod http_rpc;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use manager::ClipboardManager;
use rofi_toys::clipboard;

fn main() {
    let app = Application::builder()
        .application_id("com.rofi-toys.clipd")
        .build();

    app.connect_activate(|app| {
        let clipboard = gtk::Clipboard::get(&gtk::gdk::SELECTION_CLIPBOARD);
        let manager = ClipboardManager::new(32, clipboard::get_clipd_listen_path());

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
