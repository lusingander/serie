use std::path::PathBuf;

use serde::Deserialize;

const APP_DIR_NAME: &str = "serie";
const CONFIG_FILE_NAME: &str = "config.toml";

const DEFAULT_LIST_SUBJECT_MIN_WIDTH: u16 = 20;
const DEFAULT_LIST_DATE_FORMAT: &str = "%Y-%m-%d";
const DEFAULT_LIST_DATE_WIDTH: u16 = 10;
const DEFAULT_LIST_DATE_LOCAL: bool = true;
const DEFAULT_LIST_NAME_WIDTH: u16 = 20;
const DEFAULT_DETAIL_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";
const DEFAULT_DETAIL_DATE_LOCAL: bool = true;

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub ui: UiConfig,
    pub custom_keybind_path: Option<PathBuf>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct UiConfig {
    #[serde(default)]
    pub list: UiListConfig,
    #[serde(default)]
    pub detail: UiDetailConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UiListConfig {
    #[serde(default = "ui_list_subject_min_width_default")]
    pub subject_min_width: u16,
    #[serde(default = "ui_list_date_format_default")]
    pub date_format: String,
    #[serde(default = "ui_list_date_width_default")]
    pub date_width: u16,
    #[serde(default = "ui_list_date_local_default")]
    pub date_local: bool,
    #[serde(default = "ui_list_name_width_default")]
    pub name_width: u16,
}

impl Default for UiListConfig {
    fn default() -> Self {
        Self {
            subject_min_width: ui_list_subject_min_width_default(),
            date_format: ui_list_date_format_default(),
            date_width: ui_list_date_width_default(),
            date_local: ui_list_date_local_default(),
            name_width: ui_list_name_width_default(),
        }
    }
}

fn ui_list_subject_min_width_default() -> u16 {
    DEFAULT_LIST_SUBJECT_MIN_WIDTH
}

fn ui_list_date_format_default() -> String {
    DEFAULT_LIST_DATE_FORMAT.to_string()
}

fn ui_list_date_width_default() -> u16 {
    DEFAULT_LIST_DATE_WIDTH
}

fn ui_list_date_local_default() -> bool {
    DEFAULT_LIST_DATE_LOCAL
}

fn ui_list_name_width_default() -> u16 {
    DEFAULT_LIST_NAME_WIDTH
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UiDetailConfig {
    #[serde(default = "ui_detail_date_format_default")]
    pub date_format: String,
    #[serde(default = "ui_detail_date_local_default")]
    pub date_local: bool,
}

impl Default for UiDetailConfig {
    fn default() -> Self {
        Self {
            date_format: ui_detail_date_format_default(),
            date_local: ui_detail_date_local_default(),
        }
    }
}

fn ui_detail_date_format_default() -> String {
    DEFAULT_DETAIL_DATE_FORMAT.to_string()
}

fn ui_detail_date_local_default() -> bool {
    DEFAULT_DETAIL_DATE_LOCAL
}

impl Config {
    pub fn load() -> Config {
        let path = xdg::BaseDirectories::with_prefix(APP_DIR_NAME)
            .unwrap()
            .get_config_file(CONFIG_FILE_NAME);
        if path.exists() {
            let content = std::fs::read_to_string(path).unwrap();
            toml::from_str(&content).unwrap()
        } else {
            Config::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let actual = Config::default();
        let expected = Config {
            ui: UiConfig {
                list: UiListConfig {
                    subject_min_width: 20,
                    date_format: "%Y-%m-%d".into(),
                    date_width: 10,
                    date_local: true,
                    name_width: 20,
                },
                detail: UiDetailConfig {
                    date_format: "%Y-%m-%d %H:%M:%S %z".into(),
                    date_local: true,
                },
            },
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_config_complete_toml() {
        let toml = r#"
            [ui.list]
            subject_min_width = 40
            date_format = "%Y/%m/%d"
            date_width = 20
            date_local = false
            name_width = 30
            [ui.detail]
            date_format = "%Y/%m/%d %H:%M:%S"
            date_local = false
        "#;
        let actual: Config = toml::from_str(toml).unwrap();
        let expected = Config {
            ui: UiConfig {
                list: UiListConfig {
                    subject_min_width: 40,
                    date_format: "%Y/%m/%d".into(),
                    date_width: 20,
                    date_local: false,
                    name_width: 30,
                },
                detail: UiDetailConfig {
                    date_format: "%Y/%m/%d %H:%M:%S".into(),
                    date_local: false,
                },
            },
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_config_partial_toml() {
        let toml = r#"
            [ui.list]
            date_format = "%Y/%m/%d"
        "#;
        let actual: Config = toml::from_str(toml).unwrap();
        let expected = Config {
            ui: UiConfig {
                list: UiListConfig {
                    subject_min_width: 20,
                    date_format: "%Y/%m/%d".into(),
                    date_width: 10,
                    date_local: true,
                    name_width: 20,
                },
                detail: UiDetailConfig {
                    date_format: "%Y-%m-%d %H:%M:%S %z".into(),
                    date_local: true,
                },
            },
        };
        assert_eq!(actual, expected);
    }
}
