use std::{collections::HashMap, fs};

use crate::rofi::RofiPluginError;

pub fn storage_save_to_file<T: serde::Serialize>(object: T, name: &str) -> anyhow::Result<()> {
    let data_dir = dirs::data_dir().unwrap().join("rofi-toys");
    fs::create_dir_all(&data_dir)?;

    let storage_path = data_dir.join("storage.json");
    let storage_data = match fs::read_to_string(&storage_path) {
        Ok(x) => x,
        Err(_) => "{}".to_string(),
    };
    let mut map: HashMap<String, serde_json::Value> = serde_json::from_str(&storage_data)?;

    map.insert(name.to_string(), serde_json::to_value(object).unwrap());

    let storage_data = serde_json::to_string(&map).unwrap();
    Ok(fs::write(storage_path, storage_data)?)
}

pub fn storage_restore_from_file<T: serde::de::DeserializeOwned>(name: &str) -> anyhow::Result<T> {
    let data_dir = dirs::data_dir().unwrap().join("rofi-toys");
    fs::create_dir_all(&data_dir)?;

    let storage_path = data_dir.join("storage.json");
    let storage_data = fs::read_to_string(storage_path)?;
    let map: HashMap<String, serde_json::Value> = serde_json::from_str(&storage_data)?;

    match map.get(name) {
        Some(x) => Ok(serde_json::from_value(x.clone())?),
        None => Err(RofiPluginError::new(&format!("no such key:{} in map",name)).into()),
    }
}

pub fn config_save_to_file<T: serde::Serialize>(object: T, name: &str) -> anyhow::Result<()> {
    let data_dir = dirs::data_dir().unwrap().join("rofi-toys");
    fs::create_dir_all(&data_dir)?;

    let config_path = data_dir.join("config.toml");
    let config_data = match fs::read_to_string(&config_path) {
        Ok(x) => x,
        Err(_) => "".to_string(),
    };
    let mut map: HashMap<String, toml::Value> = toml::from_str(&config_data)?;

    map.insert(name.to_string(), toml::Value::try_from(object).unwrap());

    let config_data = toml::to_string(&map).unwrap();
    Ok(fs::write(config_path, config_data)?)
}

pub fn config_restore_from_file<T: serde::de::DeserializeOwned>(name: &str) -> anyhow::Result<T> {
    let data_dir = dirs::data_dir().unwrap().join("rofi-toys");
    fs::create_dir_all(&data_dir)?;

    let storage_path = data_dir.join("config.toml");
    let storage_data = fs::read_to_string(storage_path)?;
    let map: HashMap<String, toml::Value> = toml::from_str(&storage_data)?;

    match map.get(name) {
        Some(x) => Ok(toml::Value::try_into(x.clone())?),
        None => Err(RofiPluginError::new(&format!("no such key:{} in map",name)).into()),
    }
}
