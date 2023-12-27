use std::collections::HashMap;

use rofi_toys::clipboard;
use rofi_toys::file;
use rofi_toys::rofi::RofiPlugin;

#[derive(serde::Serialize, serde::Deserialize)]
struct Notes {
    contents: HashMap<String, String>,
}

fn serialize_notes(notes: &Notes) -> anyhow::Result<()> {
    file::storage_save_to_file(notes, "notes")
}

fn deserialize_notes() -> Notes {
    match file::storage_restore_from_file("notes") {
        Ok(x) => x,
        Err(_) => Notes {
            contents: HashMap::new(),
        },
    }
}

pub fn list_notes(rofi: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let notes = deserialize_notes();

    rofi.add_menu_entry_with_params(
        "[add_from_clipboard]",
        save_current_clipboard_text_to_notes,
        vec![],
    );

    for (k, v) in notes.contents {
        rofi.add_menu_entry_with_params(&format!("{k} {v}"), set_clipboard, vec![v.clone()])
    }

    Ok(())
}

fn save_current_clipboard_text_to_notes(_: &RofiPlugin, params: Vec<String>) -> anyhow::Result<()> {
    let clipboard_text = clipboard::clipboard_get_text();

    let mut notes = deserialize_notes();
    notes.contents.insert(params[0].clone(), clipboard_text);
    serialize_notes(&notes)
}

fn set_clipboard(_: &RofiPlugin, params: Vec<String>) -> anyhow::Result<()> {
    clipboard::clipboard_set_text(&params[0]);

    Ok(())
}

fn main() {
    let mut rofi = RofiPlugin::new();

    rofi.register_entrypoint(list_notes);

    rofi.register_callback_with_params(set_clipboard, vec![String::from("content")]);
    rofi.register_callback_with_params(
        save_current_clipboard_text_to_notes,
        vec![String::from("key")],
    );

    rofi.run();
}
