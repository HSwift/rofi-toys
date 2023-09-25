use std::collections::HashMap;

use base64::engine::general_purpose as base64_engine;
use base64::Engine;
use md5::{Digest, Md5};
use pyo3::Python;
use rand::Rng;
use rofi_toys::clipboard;
use rofi_toys::rofi::{RofiPlugin, RofiPluginError};
use sha2::Sha256;
use uuid::Uuid;

fn get_string_length(str: &str) -> usize {
    str.chars().count()
}

fn len(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    clipboard::clipboard_set_text(&get_string_length(&input).to_string());

    Ok(())
}

fn base64_encoding(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    clipboard::clipboard_set_text(&base64_engine::STANDARD.encode(&input));

    Ok(())
}

fn base64_decoding(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    let decode_result: Vec<u8> = base64_engine::STANDARD.decode(&input)?;
    clipboard::clipboard_set_text(&String::from_utf8_lossy(&decode_result).to_string());

    Ok(())
}

fn base64_url_encoding(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    clipboard::clipboard_set_text(&base64_engine::URL_SAFE.encode(&input));

    Ok(())
}

fn base64_url_decoding(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    let decode_result = base64_engine::URL_SAFE.decode(&input)?;
    clipboard::clipboard_set_text(&String::from_utf8_lossy(&decode_result).to_string());

    Ok(())
}

fn hex_encoding(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    clipboard::clipboard_set_text(&hex::encode(input.as_bytes()));

    Ok(())
}

fn hex_decoding(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    let decode_result = hex::decode(&input)?;
    clipboard::clipboard_set_text(&String::from_utf8_lossy(&decode_result).to_string());

    Ok(())
}

fn url_encoding(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    clipboard::clipboard_set_text(&urlencoding::encode(&input));

    Ok(())
}

fn url_all_encoding(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    let encode_result: String = input
        .as_bytes()
        .iter()
        .fold(String::new(), |mut acc, curr| {
            acc.push_str(percent_encoding::percent_encode_byte(*curr));
            acc
        });
    clipboard::clipboard_set_text(&encode_result);

    Ok(())
}

fn url_decoding(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    let decode_result = urlencoding::decode(&input)?;
    clipboard::clipboard_set_text(&decode_result);

    Ok(())
}

fn html_encoding(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    clipboard::clipboard_set_text(&html_escape::encode_unquoted_attribute(&input));

    Ok(())
}

fn html_decoding(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    clipboard::clipboard_set_text(&html_escape::decode_html_entities(&input));

    Ok(())
}

fn unicode_encoding(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
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
    clipboard::clipboard_set_text(&encode_result);

    Ok(())
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

fn unicode_decoding(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    if let Some(decode_result) = unicode_decodeing_helper(input) {
        clipboard::clipboard_set_text(&decode_result);
        Ok(())
    } else {
        Err(RofiPluginError::new("unicode decoding failed").into())
    }
}

fn pyeval(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();

    let result = Python::with_gil(|py| py.eval(&input, None, None).map(|v| v.to_string()));
    clipboard::clipboard_set_text(&result?);

    Ok(())
}

fn pyeval_input(_: &RofiPlugin, params: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();

    let result = Python::with_gil(|py| {
        let locals = pyo3::types::PyDict::new(py);
        locals.set_item("input", &input).unwrap();

        py.eval(&params[0], None, Some(locals))
            .map(|v| v.to_string())
    });
    clipboard::clipboard_set_text(&result?);

    Ok(())
}

fn pyexec(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();

    let result = Python::with_gil(|py| {
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

        (result, locals.get_item("__output").map(|v| v.to_string()))
    });

    result.0?;

    if let Some(result) = result.1 {
        clipboard::clipboard_set_text(&result);

        Ok(())
    } else {
        Err(RofiPluginError::new("can't get output var in locals").into())
    }
}

fn replace(_: &RofiPlugin, params: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    clipboard::clipboard_set_text(&input.replace(&params[0], &params[1]));

    Ok(())
}

fn regex_replace(_: &RofiPlugin, params: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    let regex = regex::Regex::new(&params[0])?;
    clipboard::clipboard_set_text(&regex.replace_all(&input, &params[1]));

    Ok(())
}

fn remove(_: &RofiPlugin, params: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    clipboard::clipboard_set_text(&input.replace(&params[0], ""));

    Ok(())
}

fn regex_remove(_: &RofiPlugin, params: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    let regex = regex::Regex::new(&params[0])?;
    clipboard::clipboard_set_text(&regex.replace_all(&input, ""));

    Ok(())
}

fn uuid(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    clipboard::clipboard_set_text(&Uuid::new_v4().to_string());

    Ok(())
}

fn random(_: &RofiPlugin, params: Vec<String>) -> anyhow::Result<()> {
    let length = params[0].parse::<usize>()?;
    let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".as_bytes();
    let mut rng = rand::thread_rng();
    let mut result: Vec<u8> = Vec::new();

    for _ in 0..length {
        result.push(charset[rng.gen_range(0..charset.len())]);
    }
    clipboard::clipboard_set_text(&String::from_utf8_lossy(result.as_slice()));

    Ok(())
}

fn json_format(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    let value = serde_json::from_str::<serde_json::Value>(&input)?;
    clipboard::clipboard_set_text(&serde_json::to_string_pretty(&value).unwrap());

    Ok(())
}

fn md5(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();

    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());

    let hash = hasher.finalize();
    clipboard::clipboard_set_text(&hex::encode(&hash));

    Ok(())
}

fn sha256(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();

    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());

    let hash = hasher.finalize();
    clipboard::clipboard_set_text(&hex::encode(&hash));

    Ok(())
}

fn upper(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    clipboard::clipboard_set_text(&input.to_uppercase());

    Ok(())
}

fn lower(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    clipboard::clipboard_set_text(&input.to_lowercase());

    Ok(())
}

fn substring(_: &RofiPlugin, params: Vec<String>) -> anyhow::Result<()> {
    let (start, mut end) = (params[0].parse::<usize>()?, params[1].parse::<usize>()?);
    let input = clipboard::clipboard_get_text();
    let input_length = input.len();
    if start < input_length && start < end {
        if end > input_length {
            end = input_length;
        }

        clipboard::clipboard_set_text(&input[start..end]);
        Ok(())
    } else {
        Err(RofiPluginError::new("invalid start/end").into())
    }
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

fn qs_to_json(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    let mut qs_value: HashMap<String, DuplicateQSValue> = HashMap::new();

    for kv in form_urlencoded::parse(input.as_bytes()) {
        qs_value
            .entry(kv.0.to_string())
            .or_default()
            .push(kv.1.to_string());
    }

    clipboard::clipboard_set_text(&serde_json::to_string(&qs_value).unwrap());

    Ok(())
}

fn ord(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    if let Some(c) = input.chars().next() {
        clipboard::clipboard_set_text(&(c as u32).to_string());
        Ok(())
    } else {
        Err(RofiPluginError::new("invalid input for ord").into())
    }
}

fn chr(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    let codepoint = input.parse::<usize>()?;
    if let Some(c) = char::from_u32(codepoint as u32) {
        let mut s = String::new();
        s.push(c);
        clipboard::clipboard_set_text(&s);

        Ok(())
    } else {
        Err(RofiPluginError::new("invalid codepoint").into())
    }
}

fn entrypoint(rofi: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let input = clipboard::clipboard_get_text();
    let input_length = get_string_length(&input);

    let mut input = input.chars().take(100).collect::<String>();
    if input_length > 100 {
        input.push_str("…");
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

    Ok(())
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
