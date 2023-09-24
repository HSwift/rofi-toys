use rofi_toys::clipboard;
use rofi_toys::rofi::{RofiPlugin, RofiPluginError};

pub fn list_clipboard(rofi: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let clipboard_datas = clipboard::clipboard_list();
    if clipboard_datas.is_empty() {
        return Err(RofiPluginError::new("clipboard is empty").into());
    }

    for (idx, entry) in clipboard_datas.iter().enumerate() {
        rofi.add_menu_entry_with_params(&entry, set_clipboard, vec![idx.to_string()])
    }

    Ok(())
}

pub fn set_clipboard(_: &RofiPlugin, params: Vec<String>) -> anyhow::Result<()> {
    clipboard::clipboard_set_by_idx(params[0].parse::<usize>()?);

    Ok(())
}

fn main() {
    let mut rofi = RofiPlugin::new();

    rofi.register_entrypoint(list_clipboard);

    rofi.register_callback_with_params(set_clipboard, vec![String::from("idx")]);

    rofi.run();
}
