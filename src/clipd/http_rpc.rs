use crate::{manager, stroage};
use actix_web::{middleware, rt, web, App, HttpServer, Responder};
use rofi_toys::clipboard;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ClipboardServer {
    clipboard_tx: gtk::glib::Sender<manager::ClipboardData>,
}

unsafe impl Send for ClipboardServer {}

impl ClipboardServer {
    pub fn start_server(listen_path: String, storage: Arc<Mutex<stroage::ClipboardStorage>>) {
        let (tx, rx) =
            gtk::glib::MainContext::channel::<manager::ClipboardData>(gtk::glib::Priority::DEFAULT);
        let clipboard = gtk::Clipboard::get(&gtk::gdk::SELECTION_CLIPBOARD);

        rx.attach(None, move |data| {
            data.submit_to_clipboard(&clipboard);
            return gtk::glib::ControlFlow::Continue;
        });

        let state = ClipboardServer { clipboard_tx: tx };

        std::thread::spawn(move || {
            rt::System::new()
                .block_on(
                    HttpServer::new(move || {
                        App::new()
                            .wrap(middleware::Logger::default())
                            .route(
                                clipboard::CLIPBOARD_LIST,
                                web::get().to(ClipboardServer::list_clipboard),
                            )
                            .route(
                                clipboard::CLIPBOARD_SET_BY_IDX,
                                web::post().to(ClipboardServer::set_by_idx),
                            )
                            .route(
                                clipboard::CLIPBOARD_SET_TEXT,
                                web::post().to(ClipboardServer::set_text),
                            )
                            .route(
                                clipboard::CLIPBOARD_GET_LATEST_TEXT,
                                web::get().to(ClipboardServer::get_latest_text),
                            )
                            .app_data(web::Data::from(storage.clone()))
                            .app_data(web::Data::new(Mutex::new(state.clone())))
                    })
                    // 需要 disable_signals, 不然 SIGTERM 不会立即退出, 这里不需要 Graceful shutdown
                    .disable_signals() 
                    .bind_uds(listen_path)
                    .expect("clipboard server listen failed")
                    .workers(1)
                    .run(),
                )
                .expect("clipboard server run failed");
        });
    }

    async fn list_clipboard(
        stroage: web::Data<Mutex<stroage::ClipboardStorage>>,
    ) -> impl Responder {
        let stroage = stroage.lock().unwrap();
        return web::Json(clipboard::ClipboardListResult {
            result: stroage
                .list()
                .iter()
                .map(|e| e.to_string_with_limit(512))
                .collect(),
        });
    }

    async fn set_by_idx(
        request: web::Json<clipboard::ClipboardSetByIdxRequest>,
        stroage: web::Data<Mutex<stroage::ClipboardStorage>>,
        state: web::Data<Mutex<ClipboardServer>>,
    ) -> impl Responder {
        let mut stroage = stroage.lock().unwrap();
        let state = state.lock().unwrap();

        if let Some(data) = stroage.move_to_front(request.idx) {
            state
                .clipboard_tx
                .send(data.to_owned())
                .expect("clipboard data send failed");
        }

        return web::Json({});
    }

    async fn set_text(
        request: web::Json<clipboard::ClipboardSetTextRequest>,
        stroage: web::Data<Mutex<stroage::ClipboardStorage>>,
        state: web::Data<Mutex<ClipboardServer>>,
    ) -> impl Responder {
        let mut stroage = stroage.lock().unwrap();
        let state = state.lock().unwrap();

        stroage.insert_data(crate::manager::ClipboardData::Text(request.text.clone()));
        state
            .clipboard_tx
            .send(stroage.get_latest_data().unwrap().to_owned())
            .expect("clipboard data send failed");
        return web::Json({});
    }

    async fn get_latest_text(
        stroage: web::Data<Mutex<stroage::ClipboardStorage>>,
    ) -> impl Responder {
        let stroage = stroage.lock().unwrap();
        let text = if let Some(data) = stroage.get_latest_data() {
            data.to_string()
        } else {
            String::new()
        };

        return web::Json(clipboard::ClipboardGetLatestTextResult { result: text });
    }
}
