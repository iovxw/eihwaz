use std;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use glib;
use serde_json;

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Json(serde_json::Error),
    InvalidConfig,
    UnknownItemType,
    DuplicateKey,
}

type ConfigResult<T> = Result<T, ConfigError>;

fn get_user_config_dir() -> String {
    if let Some(dir) = glib::utils::get_user_config_dir() {
        dir
    } else {
        if let Some(path) = std::env::home_dir() {
            let home_path = path.to_str().unwrap().to_string();
            home_path + "/.config"
        } else {
            panic!("can not get user config dir")
        }
    }
}

fn parse_item(item: &serde_json::Value) -> ConfigResult<::Item> {
    let item = item.as_object().ok_or(ConfigError::InvalidConfig)?;
    let key = item.get("key")
        .ok_or(ConfigError::InvalidConfig)?
        .as_str()
        .ok_or(ConfigError::InvalidConfig)?
        .chars()
        .next()
        .ok_or(ConfigError::InvalidConfig)?;
    let text = item.get("text")
        .ok_or(ConfigError::InvalidConfig)?
        .as_str()
        .ok_or(ConfigError::InvalidConfig)?
        .to_string();
    let raw_value = item.get("value").ok_or(ConfigError::InvalidConfig)?;
    let value = match item.get("type")
        .ok_or(ConfigError::InvalidConfig)?
        .as_str()
        .ok_or(ConfigError::InvalidConfig)? {
        "file" => {
            ::ItemValue::File(raw_value.as_str()
                .ok_or(ConfigError::InvalidConfig)?
                .to_string())
        }
        "command" => {
            ::ItemValue::Command(raw_value.as_str()
                .ok_or(ConfigError::InvalidConfig)?
                .to_string())
        }
        "application" => {
            ::ItemValue::Command(raw_value.as_str()
                .ok_or(ConfigError::InvalidConfig)?
                .to_string())
        }
        "index" => ::ItemValue::Index(parse_items(raw_value)?),
        _ => return Err(ConfigError::UnknownItemType),
    };
    Ok(::Item {
        key: key,
        text: text,
        value: value,
    })
}

fn items_key_duplicate(items: &Vec<::Item>) -> bool {
    let mut key_list: Vec<char> = Vec::with_capacity(items.len());
    for item in items {
        if key_list.contains(&item.key) {
            return true;
        }
        key_list.push(item.key);
    }
    false
}

fn parse_items(items: &serde_json::Value) -> ConfigResult<Vec<::Item>> {
    let items = items.as_array().ok_or(ConfigError::InvalidConfig)?;
    let items = items.iter()
        .map(parse_item)
        .collect::<ConfigResult<Vec<::Item>>>()?;
    if items_key_duplicate(&items) {
        Err(ConfigError::DuplicateKey)
    } else {
        Ok(items)
    }
}

fn read_config(config_file: &Path) -> ConfigResult<Vec<::Item>> {
    let file = File::open(config_file).map_err(ConfigError::Io)?;
    let config: serde_json::Value = serde_json::from_reader(file).map_err(ConfigError::Json)?;
    parse_items(&config)
}

pub fn load_config() -> ConfigResult<Vec<::Item>> {
    let config_dir = get_user_config_dir() + "/eihwaz";
    try!(fs::create_dir_all(&config_dir).map_err(ConfigError::Io));

    let config_file = config_dir + "/config.json";
    let config_file = Path::new(&config_file);
    if !config_file.exists() {
        let mut f = File::create(config_file).map_err(ConfigError::Io)?;
        f.write_all(b"[
   {
      \"key\":\"a\",
      \"type\":\"index\",
      \"text\":\"test\",
      \"value\":[
         {
            \"key\":\"b\",
            \"type\":\"command\",
            \"text\":\"run `pwd`\",
            \"value\":\"pwd\"
         }
      ]
   }
]")
            .map_err(ConfigError::Io)?;
    }
    read_config(&config_file)
}
