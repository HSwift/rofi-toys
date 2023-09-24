use http::request::Builder;
use isahc::{config::Dialer, prelude::*};
use serde::{Deserialize, Serialize};

pub const CLIPBOARD_LIST: &str = "/list";
pub const CLIPBOARD_SET_BY_IDX: &str = "/set_by_idx";

pub const CLIPBOARD_GET_LATEST_TEXT: &str = "/get_latest_text";
pub const CLIPBOARD_SET_TEXT: &str = "/set_text";

fn get_tmp_dir() -> String {
    // 先尝试 XDG_RUNTIME_DIR (/run/user/1000), 不行用 /tmp/
    if let Ok(tmp_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let md = std::fs::metadata(&tmp_dir);
        if let Ok(md) = md {
            let permissions = md.permissions();
            if !permissions.readonly() {
                return tmp_dir;
            }
        }
    }

    return std::env::temp_dir().to_string_lossy().to_string();
}

pub fn get_clipd_listen_path() -> String {
    return format!("{}/clipd.sock", get_tmp_dir());
}

pub fn get_clipd_request(path: &str) -> Builder {
    Builder::new()
        .uri(format!("http://localhost{}", path))
        .dial(Dialer::unix_socket(get_clipd_listen_path()))
}

#[derive(Serialize, Deserialize)]
pub struct ClipboardGetLatestTextResult {
    pub result: String,
}

#[derive(Serialize, Deserialize)]
pub struct ClipboardSetTextRequest {
    pub text: String,
}

#[derive(Serialize, Deserialize)]
pub struct ClipboardListResult {
    pub result: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ClipboardSetByIdxRequest {
    pub idx: usize,
}

pub fn clipboard_get_text() -> String {
    let request = get_clipd_request(CLIPBOARD_GET_LATEST_TEXT)
        .method(http::Method::GET)
        .body(())
        .unwrap();
    let response: ClipboardGetLatestTextResult = serde_json::from_slice(
        request
            .send()
            .expect("sending request failed")
            .bytes()
            .expect("reading response failed")
            .as_ref(),
    )
    .expect("recv response failed");
    return response.result;
}

pub fn clipboard_set_text(text: &str) {
    let request = get_clipd_request(CLIPBOARD_SET_TEXT)
        .method(http::Method::POST)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(
            serde_json::to_vec(&ClipboardSetTextRequest {
                text: text.to_owned(),
            })
            .unwrap(),
        )
        .unwrap();
    request.send().expect("sending request failed");
}

pub fn clipboard_list() -> Vec<String> {
    let request = get_clipd_request(CLIPBOARD_LIST)
        .method(http::Method::GET)
        .body(())
        .unwrap();
    let response: ClipboardListResult = serde_json::from_slice(
        request
            .send()
            .expect("sending request failed")
            .bytes()
            .expect("reading response failed")
            .as_ref(),
    )
    .expect("recv response failed");
    return response.result;
}

pub fn clipboard_set_by_idx(idx: usize) {
    let request = get_clipd_request(CLIPBOARD_SET_BY_IDX)
        .method(http::Method::POST)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_vec(&ClipboardSetByIdxRequest { idx }).unwrap())
        .unwrap();
    request.send().expect("sending request failed");
}
