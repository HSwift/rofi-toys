use std::collections::HashMap;

use base64::engine::general_purpose as base64_engine;
use base64::Engine;
use md5::{Digest, Md5};
use pyo3::Python;
use rand::Rng;
use rofi_toys::clipboard;
use rofi_toys::rofi::RofiPlugin;
use sha2::Sha256;
use uuid::Uuid;

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

fn base64_url_decoding(rofi: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    let decode_result = base64_engine::URL_SAFE.decode(&input);
    if let Ok(decode_result) = decode_result {
        clipboard::set_clipboard_text(&String::from_utf8_lossy(&decode_result).to_string());
    } else {
        rofi.show_error("base64 decoding failed");
    }
}

fn hex_encoding(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    clipboard::set_clipboard_text(&hex::encode(input.as_bytes()));
}

fn hex_decoding(rofi: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    let decode_result = hex::decode(&input);
    if let Ok(decode_result) = decode_result {
        clipboard::set_clipboard_text(&String::from_utf8_lossy(&decode_result).to_string());
    } else {
        rofi.show_error("hex decoding failed");
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

fn url_decoding(rofi: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    let decode_result = urlencoding::decode(&input);
    if let Ok(decode_result) = decode_result {
        clipboard::set_clipboard_text(&decode_result);
    } else {
        rofi.show_error("url decoding failed");
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

fn unicode_decoding(rofi: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    if let Some(decode_result) = unicode_decodeing_helper(input) {
        clipboard::set_clipboard_text(&decode_result);
    } else {
        rofi.show_error("unicode decoding failed");
    }
}

fn pyeval(rofi: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();

    Python::with_gil(|py| {
        let result = py.eval(&input, None, None);

        match result {
            Ok(result) => {
                clipboard::set_clipboard_text(&result.to_string());
            }
            Err(err) => {
                rofi.show_error(&err.to_string());
            }
        }
    });
}

fn pyeval_input(rofi: &RofiPlugin, params: Vec<String>) {
    let input = clipboard::get_clipboard_text();

    Python::with_gil(|py| {
        let locals = pyo3::types::PyDict::new(py);
        locals.set_item("input", &input).unwrap();

        let result = py.eval(&params[0], None, Some(locals));

        match result {
            Ok(result) => {
                clipboard::set_clipboard_text(&result.to_string());
            }
            Err(err) => {
                rofi.show_error(&err.to_string());
            }
        }
    });
}

fn pyexec(rofi: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();

    Python::with_gil(|py| {
        let locals = pyo3::types::PyDict::new(py);
        locals.set_item("__code", &input).unwrap();

        let result = py.run(
            r#"
from io import StringIO
from contextlib import redirect_stdout

__stdout = StringIO()
with redirect_stdout(__stdout):
    exec(__code)
__output = __stdout.getvalue()
"#,
            None,
            Some(locals),
        );
        match result {
            Ok(_) => {
                if let Some(result) = locals.get_item("__output") {
                    clipboard::set_clipboard_text(&result.to_string());
                }
            }
            Err(err) => {
                rofi.show_error(&err.to_string());
            }
        }
    });
}

fn replace(_: &RofiPlugin, params: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    clipboard::set_clipboard_text(&input.replace(&params[0], &params[1]));
}

fn regex_replace(rofi: &RofiPlugin, params: Vec<String>) {
    let input = clipboard::get_clipboard_text();

    match regex::Regex::new(&params[0]) {
        Ok(regex) => {
            clipboard::set_clipboard_text(&regex.replace_all(&input, &params[1]));
        }
        Err(err) => {
            rofi.show_error(&err.to_string());
        }
    };
}

fn remove(_: &RofiPlugin, params: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    clipboard::set_clipboard_text(&input.replace(&params[0], ""));
}

fn regex_remove(rofi: &RofiPlugin, params: Vec<String>) {
    let input = clipboard::get_clipboard_text();

    match regex::Regex::new(&params[0]) {
        Ok(regex) => {
            clipboard::set_clipboard_text(&regex.replace_all(&input, ""));
        }
        Err(err) => {
            rofi.show_error(&err.to_string());
        }
    };
}

fn uuid(_: &RofiPlugin, _: Vec<String>) {
    clipboard::set_clipboard_text(&Uuid::new_v4().to_string());
}

fn random(rofi: &RofiPlugin, params: Vec<String>) {
    if let Ok(length) = params[0].parse::<usize>() {
        let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".as_bytes();
        let mut rng = rand::thread_rng();
        let mut result: Vec<u8> = Vec::new();

        for _ in 0..length {
            result.push(charset[rng.gen_range(0..charset.len())]);
        }
        clipboard::set_clipboard_text(&String::from_utf8_lossy(result.as_slice()));
    } else {
        rofi.show_error("invalid length");
    }
}

fn json_format(rofi: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();

    match serde_json::from_str::<serde_json::Value>(&input) {
        Ok(value) => {
            clipboard::set_clipboard_text(&serde_json::to_string_pretty(&value).unwrap());
        }
        Err(err) => {
            rofi.show_error(&err.to_string());
        }
    }
}

fn md5(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();

    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());

    let hash = hasher.finalize();
    clipboard::set_clipboard_text(&hex::encode(&hash));
}

fn sha256(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();

    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());

    let hash = hasher.finalize();
    clipboard::set_clipboard_text(&hex::encode(&hash));
}

fn upper(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    clipboard::set_clipboard_text(&input.to_uppercase());
}

fn lower(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    clipboard::set_clipboard_text(&input.to_lowercase());
}

fn substring(rofi: &RofiPlugin, params: Vec<String>) {
    if let (Ok(start), Ok(mut end)) = (params[0].parse::<usize>(), params[1].parse::<usize>()) {
        let input = clipboard::get_clipboard_text();
        let input_length = input.len();
        if start < input_length && start < end {
            if end > input_length {
                end = input_length;
            }

            clipboard::set_clipboard_text(&input[start..end]);
            return;
        }
    }
    rofi.show_error("invalid start/end");
}

#[derive(Default)]
struct DuplicateQSValue(Vec<String>);

impl DuplicateQSValue {
    fn push(&mut self, s: String) {
        self.0.push(s);
    }
}

impl serde::Serialize for DuplicateQSValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.0.len() > 1 {
            self.0.serialize(serializer)
        } else {
            self.0[0].serialize(serializer)
        }
    }
}

fn qs_to_json(_: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    let mut qs_value: HashMap<String, DuplicateQSValue> = HashMap::new();

    for kv in form_urlencoded::parse(input.as_bytes()) {
        qs_value
            .entry(kv.0.to_string())
            .or_default()
            .push(kv.1.to_string());
    }

    clipboard::set_clipboard_text(&serde_json::to_string(&qs_value).unwrap());
}

fn ord(rofi: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();
    if let Some(c) = input.chars().next() {
        clipboard::set_clipboard_text(&(c as u32).to_string());
    } else {
        rofi.show_error("invalid input for ord");
    }
}

fn chr(rofi: &RofiPlugin, _: Vec<String>) {
    let input = clipboard::get_clipboard_text();

    if let Ok(start) = input.parse::<usize>() {
        if let Some(c) = char::from_u32(start as u32) {
            let mut s = String::new();
            s.push(c);
            clipboard::set_clipboard_text(&s);
            return;
        }
    }
    rofi.show_error("invalid codepoint");
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
    rofi.add_menu_entry("pyeval", pyeval);
    rofi.add_menu_entry("pyeval_input", pyeval_input);
    rofi.add_menu_entry("pyexec", pyexec);
    rofi.add_menu_entry("replace", replace);
    rofi.add_menu_entry("regex_replace", regex_replace);
    rofi.add_menu_entry("remove", remove);
    rofi.add_menu_entry("regex_remove", regex_remove);
    rofi.add_menu_entry("uuid", uuid);
    rofi.add_menu_entry("random", random);
    rofi.add_menu_entry("json_format", json_format);
    rofi.add_menu_entry("md5", md5);
    rofi.add_menu_entry("sha256", sha256);
    rofi.add_menu_entry("upper", upper);
    rofi.add_menu_entry("lower", lower);
    rofi.add_menu_entry("substring", substring);
    rofi.add_menu_entry("qs_to_json", qs_to_json);
    rofi.add_menu_entry("chr", chr);
    rofi.add_menu_entry("ord", ord);
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
    rofi.register_callback(pyeval);
    rofi.register_callback_with_params(pyeval_input, vec![String::from("code")]);
    rofi.register_callback(pyexec);
    rofi.register_callback_with_params(replace, vec![String::from("from"), String::from("to")]);
    rofi.register_callback_with_params(
        regex_replace,
        vec![String::from("from"), String::from("to")],
    );
    rofi.register_callback_with_params(remove, vec![String::from("from")]);
    rofi.register_callback_with_params(regex_remove, vec![String::from("from")]);
    rofi.register_callback(uuid);
    rofi.register_callback_with_params(random, vec![String::from("length")]);
    rofi.register_callback(json_format);
    rofi.register_callback(md5);
    rofi.register_callback(sha256);
    rofi.register_callback(upper);
    rofi.register_callback(lower);
    rofi.register_callback_with_params(substring, vec![String::from("start"), String::from("end")]);
    rofi.register_callback(qs_to_json);
    rofi.register_callback(chr);
    rofi.register_callback(ord);

    rofi.run();
}
