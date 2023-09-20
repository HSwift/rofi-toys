use std::{collections::HashMap, env::VarError};

use serde::{Deserialize, Serialize};

type RofiPluginCallback = Box<dyn Fn(&RofiPlugin, Vec<String>) + 'static>;

#[derive(Serialize, Deserialize, Debug)]
pub struct RofiPluginState {
    callback: String,
    params: Vec<String>,
    inputting_parmas: bool,
}

impl RofiPluginState {
    fn new(callback: String, params: Vec<String>) -> RofiPluginState {
        RofiPluginState {
            callback,
            params,
            inputting_parmas: false,
        }
    }

    fn empty() -> RofiPluginState {
        return RofiPluginState::new(String::new(), Vec::new());
    }

    fn parse_from_env(env_name: &str) -> Option<RofiPluginState> {
        if let Ok(raw_state) = std::env::var(env_name) {
            let decode_result: Result<RofiPluginState, serde_json::Error> =
                serde_json::from_str(&raw_state);
            return Some(decode_result.expect("state decode failed"));
        } else {
            return None;
        }
    }
}

pub struct RofiPlugin {
    entrypoint: RofiPluginCallback,

    callbacks: HashMap<String, RofiPluginCallback>,
    callbacks_params_desc: HashMap<String, Vec<String>>,
}

impl RofiPlugin {
    pub fn new() -> RofiPlugin {
        return RofiPlugin {
            entrypoint: Box::new(|_, _| {}),
            callbacks: HashMap::new(),
            callbacks_params_desc: HashMap::new(),
        };
    }

    pub fn register_callback_with_params<F: Fn(&RofiPlugin, Vec<String>) + 'static>(
        &mut self,
        callback: F,
        params_desc: Vec<String>,
    ) {
        let callback_name = std::any::type_name::<F>().to_owned();
        self.callbacks
            .insert(callback_name.clone(), Box::new(callback));
        self.callbacks_params_desc
            .insert(callback_name, params_desc);
    }

    pub fn register_callback<F: Fn(&RofiPlugin, Vec<String>) + 'static>(&mut self, callback: F) {
        self.register_callback_with_params(callback, Vec::new());
    }

    pub fn register_entrypoint<F: Fn(&RofiPlugin, Vec<String>) + 'static>(&mut self, callback: F) {
        self.entrypoint = Box::new(callback);
    }

    pub fn run(&self) {
        let mut state = if let Some(state) = RofiPluginState::parse_from_env("ROFI_INFO") {
            // 首先从 ROFI_INFO 获取 state
            state
        } else if let Some(state) = RofiPluginState::parse_from_env("ROFI_DATA") {
            // 然后尝试 ROFI_DATA
            state
        } else {
            // 都获取不到, 调 entrypoint
            (self.entrypoint)(self, Vec::new());
            // 给一个空的 state, 这样输错后直接退出
            println!(
                "\x00data\x1f{}",
                serde_json::to_string(&RofiPluginState::empty()).unwrap()
            );
            return;
        };

        let callback_name = &state.callback;
        if let Some(callback) = self.callbacks.get(callback_name) {
            // 读取上一次输入的 input
            if state.inputting_parmas {
                if let Some(input) = std::env::args().nth(1) {
                    state.params.push(input);
                }
            }

            let params_desc = self
                .callbacks_params_desc
                .get(callback_name)
                .expect(&format!("get callback {} param desc failed", callback_name));

            // 数量足够, 调 callback, 否则继续要求用户输入更多参数
            let params_count = state.params.len();
            if params_count >= params_desc.len() {
                callback(self, state.params);
                // 清空状态
                println!(
                    "\x00data\x1f{}",
                    serde_json::to_string(&RofiPluginState::empty()).unwrap()
                );
            } else {
                let curr_required_param = &params_desc[params_count];
                state.inputting_parmas = true;
                println!("\x00prompt\x1f{}", curr_required_param);
                println!("\x00data\x1f{}", serde_json::to_string(&state).unwrap());
                println!("\x00no-custom\x1ffalse");
                println!(" \x00nonselectable\x1ftrue");
            }
            return;
        }
        // else callback 不存在
    }

    pub fn set_message(&self, msg: &str) {
        println!("\x00message\x1f{}", msg);
    }

    pub fn add_menu_entry<F: Fn(&RofiPlugin, Vec<String>) + 'static>(
        &self,
        entry: &str,
        callback: F,
    ) {
        println!(
            "{}\x00info\x1f{}",
            entry,
            serde_json::to_string(&RofiPluginState::new(
                std::any::type_name::<F>().to_owned(),
                Vec::new()
            ))
            .unwrap()
        );
    }

    pub fn add_menu_entry_with_params<F: Fn(&RofiPlugin, Vec<String>) + 'static>(
        &self,
        entry: &str,
        callback: F,
        params: Vec<String>,
    ) {
        println!(
            "{}\x00info\x1f{}",
            entry,
            serde_json::to_string(&RofiPluginState::new(
                std::any::type_name::<F>().to_owned(),
                params
            ))
            .unwrap()
        );
    }
}
