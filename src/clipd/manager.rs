use md5::{Digest, Md5};
use std::sync::{Arc, Mutex};

use crate::{http_rpc, stroage};

#[derive(Debug)]
pub struct ClipboardManager {
    storage: Arc<Mutex<stroage::ClipboardStorage>>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum ClipboardDataType {
    None = 0,
    Text = 5,
    Url = 10,
    Html = 15,
    Image = 20,
}

impl ClipboardDataType {
    pub fn from_u32(n: u32) -> ClipboardDataType {
        match n {
            5 => Self::Text,
            10 => Self::Url,
            15 => Self::Html,
            20 => Self::Image,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Eq, Clone)]
pub enum ClipboardData {
    Text(String),
    Url(Vec<String>),
    Html(String),
    Image(
        gtk::gdk_pixbuf::Pixbuf,
        String,
        chrono::DateTime<chrono::Local>,
    ),
}

impl ToString for ClipboardData {
    fn to_string(&self) -> String {
        match self {
            Self::Text(text) => text.to_owned(),
            Self::Url(html) => html.join("\n"),
            Self::Html(html) => html.to_owned(),
            Self::Image(image, hash, time) => {
                format!(
                    "[image: {}, {}, {}x{}, {}]",
                    hash,
                    time.format("%m-%d %H:%M:%S"),
                    image.width(),
                    image.height(),
                    byte_unit::Byte::from_bytes(image.byte_length() as u128)
                        .get_appropriate_unit(false)
                )
            }
        }
    }
}

impl PartialEq for ClipboardData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text(t1), Self::Text(t2)) => t1 == t2,
            (Self::Url(t1), Self::Url(t2)) => t1 == t2,
            (Self::Html(t1), Self::Html(t2)) => t1 == t2,
            (Self::Image(_, t1, _), Self::Image(_, t2, _)) => t1 == t2,
            _ => false,
        }
    }
}

impl ClipboardData {
    pub fn to_string_with_limit(&self, limit: usize) -> String {
        match self {
            Self::Text(text) => text.chars().take(limit).collect(),
            Self::Url(html) => html.join("\n").chars().take(limit).collect(),
            Self::Html(html) => html.chars().take(limit).collect(),
            Self::Image(image, hash, time) => {
                format!(
                    "[image: {}, {}, {}x{}, {}]",
                    hash,
                    time.format("%m-%d %H:%M:%S"),
                    image.width(),
                    image.height(),
                    byte_unit::Byte::from_bytes(image.byte_length() as u128)
                        .get_appropriate_unit(false)
                )
            }
        }
    }

    pub fn calc_image_hash(pixbuf: &gtk::gdk_pixbuf::Pixbuf) -> String {
        // 这里不需要关心安全性, 速度比较重要, 且长度太长了不方便人类阅读, 所以用 md5
        let mut hasher = Md5::new();
        hasher.update(pixbuf.read_pixel_bytes());
        let hash = hasher.finalize();
        hex::encode(&hash)
    }
}

unsafe impl Send for ClipboardData {}

impl ClipboardData {
    pub fn submit_to_clipboard(self, clipboard: &gtk::Clipboard) {
        match self {
            ClipboardData::Text(text) => {
                clipboard.set_text(&text);
            }
            ClipboardData::Url(urls) => {
                clipboard.set_with_data(
                    vec![
                        gtk::TargetEntry::new(
                            "text/uri-list",
                            gtk::TargetFlags::OTHER_APP,
                            ClipboardDataType::Url as u32,
                        ),
                        gtk::TargetEntry::new(
                            "UTF8_STRING",
                            gtk::TargetFlags::OTHER_APP,
                            ClipboardDataType::Text as u32,
                        ),
                    ]
                    .as_slice(),
                    move |_, data, data_type| match ClipboardDataType::from_u32(data_type) {
                        ClipboardDataType::Url => {
                            data.set_uris(
                                urls.iter()
                                    .map(|u| u.as_str())
                                    .collect::<Vec<_>>()
                                    .as_slice(),
                            );
                        }
                        ClipboardDataType::Text => {
                            data.set_text(&urls.join("\n"));
                        }
                        _ => {}
                    },
                );
            }
            ClipboardData::Html(html) => {
                clipboard.set_with_data(
                    vec![
                        gtk::TargetEntry::new(
                            "text/html",
                            gtk::TargetFlags::OTHER_APP,
                            ClipboardDataType::Html as u32,
                        ),
                        gtk::TargetEntry::new(
                            "UTF8_STRING",
                            gtk::TargetFlags::OTHER_APP,
                            ClipboardDataType::Text as u32,
                        ),
                    ]
                    .as_slice(),
                    move |_, data, data_type| match ClipboardDataType::from_u32(data_type) {
                        ClipboardDataType::Html | ClipboardDataType::Text => {
                            data.set_text(&html);
                        }
                        _ => {}
                    },
                );
            }
            ClipboardData::Image(image, _, _) => {
                clipboard.set_image(&image);
            }
        }
    }
}

impl ClipboardManager {
    pub fn new(max_size: usize, listen_path: String) -> ClipboardManager {
        let storage = Arc::new(Mutex::new(stroage::ClipboardStorage::new(max_size)));
        http_rpc::ClipboardServer::start_server(listen_path, storage.clone());

        return ClipboardManager { storage };
    }

    pub fn clipboard_on_update(&self, clipboard: gtk::Clipboard) {
        log::trace!("updating");

        let targets = clipboard.wait_for_targets();

        if let Some(atoms) = targets {
            log::trace!(
                "types, {:?}",
                atoms.iter().map(|x| x.name()).collect::<Vec<_>>()
            );

            let mut data_types: Vec<ClipboardDataType> = atoms
                .iter()
                .map(|x| {
                    let atom_name = x.name().to_lowercase();
                    match atom_name.as_str() {
                        "utf8_string" | "string" => ClipboardDataType::Text,
                        "text/uri-list" => ClipboardDataType::Url,
                        "text/html" => ClipboardDataType::Html,
                        _ if atom_name.starts_with("text/") => ClipboardDataType::Text,
                        _ if atom_name.starts_with("image/") => ClipboardDataType::Image,
                        _ => ClipboardDataType::None,
                    }
                })
                .collect();

            data_types.sort();

            // 是有效的
            let curr_data_type = if let Some(data_type) = data_types.last() {
                *data_type
            } else {
                ClipboardDataType::None
            };

            if curr_data_type == ClipboardDataType::None {
                // 如果是 None, 代表无法处理的剪贴板数据
                log::error!(
                    "unexpeced data types: {:?}",
                    atoms.iter().map(|x| x.name()).collect::<Vec<_>>()
                );
                return;
            }

            // 正常处理数据
            let data = match curr_data_type {
                ClipboardDataType::Text => clipboard
                    .wait_for_text()
                    .map(|x| ClipboardData::Text(x.to_string())),
                ClipboardDataType::Url => {
                    let urls = clipboard.wait_for_uris();
                    if urls.len() > 0 {
                        Some(ClipboardData::Url(
                            urls.iter().map(|x| x.to_string()).collect::<Vec<_>>(),
                        ))
                    } else {
                        None
                    }
                }
                ClipboardDataType::Html => clipboard
                    .wait_for_text()
                    .map(|x| ClipboardData::Html(x.to_string())),
                ClipboardDataType::Image => {
                    if let Some(pixbuf) = clipboard.wait_for_image() {
                        let image_hash = ClipboardData::calc_image_hash(&pixbuf);
                        Some(ClipboardData::Image(
                            pixbuf,
                            image_hash,
                            chrono::offset::Local::now(),
                        ))
                    } else {
                        None
                    }
                }
                _ => panic!("invalid clipboard type"),
            };

            let data = if let Some(data) = data {
                data
            } else {
                // 如果是 None, 代表获取剪贴板数据失败
                log::error!(
                    "failed to fetch clipboard data: {:?}",
                    atoms.iter().map(|x| x.name()).collect::<Vec<_>>()
                );
                return;
            };

            let mut storage = self.storage.lock().unwrap();
            storage.insert_data(data);
        } else {
            // = None, 代表目标应用关闭, 需要恢复之前的剪贴板内容
            // 目前还存在一个问题是, 恢复的设置会触发自己的 update, 造成可能的性能浪费
            // 不过在 gtk 层面好像没有地方可以解决这个问题
            log::trace!("tracing");

            let storage = self.storage.lock().unwrap();
            if let Some(lastest_data) = storage.get_latest_data() {
                let lastest_data = lastest_data.to_owned();
                lastest_data.submit_to_clipboard(&clipboard);
            }
        }
    }
}
