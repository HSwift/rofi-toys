use std::ops::ControlFlow;

use base64::engine::general_purpose as base64_engine;
use base64::Engine;
use rofi_toys::clipboard;
use rofi_toys::rofi::RofiPlugin;

fn get_string_length(str: &str) -> usize {
    str.chars().count()
}

fn len(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    clipboard::set_clipboard_text(&get_string_length(&input).to_string());
}

fn base64_encoding(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    clipboard::set_clipboard_text(&base64_engine::STANDARD.encode(&input));
}

fn base64_decoding(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    let decode_result: Vec<u8> = base64_engine::STANDARD
        .decode(&input)
        .unwrap_or_else(|_| Vec::new());
    clipboard::set_clipboard_text(&String::from_utf8_lossy(&decode_result).to_string());
}

fn base64_url_encoding(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    clipboard::set_clipboard_text(&base64_engine::URL_SAFE.encode(&input));
}

fn base64_url_decoding(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    let decode_result = base64_engine::URL_SAFE.decode(&input);
    if let Ok(decode_result) = decode_result {
        clipboard::set_clipboard_text(&String::from_utf8_lossy(&decode_result).to_string());
    }
}

fn hex_encoding(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    clipboard::set_clipboard_text(&hex::encode(input.as_bytes()));
}

fn hex_decoding(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    let decode_result = hex::decode(&input);
    if let Ok(decode_result) = decode_result {
        clipboard::set_clipboard_text(&String::from_utf8_lossy(&decode_result).to_string());
    }
}

fn url_encoding(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    clipboard::set_clipboard_text(&urlencoding::encode(&input));
}

fn url_all_encoding(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    let encode_result: String = input
        .as_bytes()
        .iter()
        .fold(String::new(), |mut acc, curr| {
            acc.push_str(percent_encoding::percent_encode_byte(*curr));
            acc
        });
    clipboard::set_clipboard_text(&encode_result);
}

fn url_decoding(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    let decode_result = urlencoding::decode(&input);
    if let Ok(decode_result) = decode_result {
        clipboard::set_clipboard_text(&decode_result);
    }
}

fn html_encoding(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    clipboard::set_clipboard_text(&html_escape::encode_unquoted_attribute(&input));
}

fn html_decoding(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    clipboard::set_clipboard_text(&html_escape::decode_html_entities(&input));
}

fn unicode_encoding(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    let encode_result = input.chars().fold(String::new(), |mut acc, c| {
        let c32 = c as u32;
        if c32 < 65536 {
            acc.push_str("\\u");
            acc.push_str(&hex::encode(&c32.to_be_bytes()[2..]));
        } else {
            acc.push_str("\\U");
            acc.push_str(&hex::encode(&c32.to_be_bytes()));
        }
        acc
    });
    clipboard::set_clipboard_text(&encode_result);
}

fn unicode_decodeing_helper(input: String) -> Option<String> {
    let mut result: Vec<u8> = Vec::new();
    let mut iter = input.bytes();

    loop {
        let raw_char = iter.next();
        let raw_char = if let Some(raw_char) = raw_char {
            raw_char
        } else {
            break;
        };

        if raw_char == b'\\' {
            let escaped_char = iter.next()?;
            match escaped_char {
                b'"' | b'\\' | b'/' => result.push(escaped_char),
                b'b' => result.push(b'\x08'),
                b'f' => result.push(b'\x0c'),
                b'n' => result.push(b'\n'),
                b'r' => result.push(b'\r'),
                b't' => result.push(b'\t'),
                b'u' => {
                    let mut unicode = String::new();

                    for _ in 0..4 {
                        let number = iter.next()?;
                        if number.is_ascii_hexdigit() {
                            unicode.push(number as char);
                        } else {
                            return None;
                        }
                    }

                    let escaped_char = char::from_u32(u32::from_str_radix(&unicode, 16).unwrap());
                    if let Some(escaped_char) = escaped_char {
                        result.extend_from_slice(escaped_char.to_string().as_bytes());
                    } else {
                        return None;
                    }
                }
                b'U' => {
                    let mut unicode = String::new();

                    for _ in 0..8 {
                        let number = iter.next()?;
                        if number.is_ascii_hexdigit() {
                            unicode.push(number as char);
                        } else {
                            return None;
                        }
                    }

                    let escaped_char = char::from_u32(u32::from_str_radix(&unicode, 16).unwrap());
                    if let Some(escaped_char) = escaped_char {
                        result.extend_from_slice(escaped_char.to_string().as_bytes());
                    } else {
                        return None;
                    }
                }
                _ => {
                    return None;
                }
            }
        } else {
            result.push(raw_char);
        }
    }
    return Some(String::from_utf8_lossy(&result).to_string());
}

fn unicode_decoding(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    if let Some(decode_result) = unicode_decodeing_helper(input) {
        clipboard::set_clipboard_text(&decode_result);
    }
}

fn entrypoint(rofi: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    let input_length = get_string_length(&input);

    let mut input = input.chars().take(50).collect::<String>();
    if input_length > 50 {
        input.push_str("...");
    }
    
    rofi.set_message(&format!(
        "<b>input: </b>{}",
        html_escape::encode_safe(&input).replace("\n", " ")
    ));

    rofi.add_menu_entry(&format!("@len: {}", input_length), len);
    rofi.add_menu_entry("base64", base64_encoding);
    rofi.add_menu_entry("base64", base64_encoding);
    rofi.add_menu_entry("base64_decode", base64_decoding);
    rofi.add_menu_entry("base64_url", base64_url_encoding);
    rofi.add_menu_entry("base64_url_decode", base64_url_decoding);
    rofi.add_menu_entry("hex", hex_encoding);
    rofi.add_menu_entry("hex_decode", hex_decoding);
    rofi.add_menu_entry("url", url_encoding);
    rofi.add_menu_entry("url_all", url_all_encoding);
    rofi.add_menu_entry("url_decode", url_decoding);
    rofi.add_menu_entry("html", html_encoding);
    rofi.add_menu_entry("html_decode", html_decoding);
    rofi.add_menu_entry("unicode", unicode_encoding);
    rofi.add_menu_entry("unicode_decode", unicode_decoding);
}

fn main() {
    let mut rofi = RofiPlugin::new();

    rofi.register_entrypoint(entrypoint);

    rofi.register_callback(len);
    rofi.register_callback(base64_encoding);
    rofi.register_callback(base64_decoding);
    rofi.register_callback(base64_url_encoding);
    rofi.register_callback(base64_url_decoding);
    rofi.register_callback(hex_encoding);
    rofi.register_callback(hex_decoding);
    rofi.register_callback(url_encoding);
    rofi.register_callback(url_all_encoding);
    rofi.register_callback(url_decoding);
    rofi.register_callback(html_encoding);
    rofi.register_callback(html_decoding);
    rofi.register_callback(unicode_encoding);
    rofi.register_callback(unicode_decoding);

    rofi.run();
}
