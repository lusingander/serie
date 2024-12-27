use std::{
    env,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::keybind::KeyBind;

const APP_DIR_NAME: &str = "serie";
const CONFIG_FILE_NAME: &str = "config.toml";
const CONFIG_FILE_ENV_NAME: &str = "SERIE_CONFIG_FILE";

const DEFAULT_LIST_SUBJECT_MIN_WIDTH: u16 = 20;
const DEFAULT_LIST_DATE_FORMAT: &str = "%Y-%m-%d";
const DEFAULT_LIST_DATE_WIDTH: u16 = 10;
const DEFAULT_LIST_DATE_LOCAL: bool = true;
const DEFAULT_LIST_NAME_WIDTH: u16 = 20;
const DEFAULT_DETAIL_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";
const DEFAULT_DETAIL_DATE_LOCAL: bool = true;
const DEFAULT_DETAIL_HEIGHT: u16 = 20;
const DEFAULT_REFS_WIDTH: u16 = 26;

const DEFAULT_GRAPH_COLOR_BRANCHES: [&str; 6] = [
    "#E06C76", "#98C379", "#E5C07B", "#61AFEF", "#C678DD", "#56B6C2",
];
const DEFAULT_GRAPH_COLOR_EDGE: &str = "#00000000";
const DEFAULT_GRAPH_COLOR_BACKGROUND: &str = "#00000000";

pub fn load() -> (UiConfig, GraphConfig, Option<KeyBind>) {
    let config = match config_file_path_from_env() {
        Some(user_path) => {
            if !user_path.exists() {
                panic!("Config file not found: {:?}", user_path);
            }
            read_config_from_path(&user_path)
        }
        None => {
            let default_path = config_file_path();
            if default_path.exists() {
                read_config_from_path(&default_path)
            } else {
                Config::default()
            }
        }
    };
    (config.ui, config.graph, config.keybind)
}

fn config_file_path_from_env() -> Option<PathBuf> {
    env::var(CONFIG_FILE_ENV_NAME).ok().map(PathBuf::from)
}

fn config_file_path() -> PathBuf {
    use directories::BaseDirs;

    BaseDirs::new()
        .expect("Couldn't get the base user directories!")
        .config_dir()
        .join(APP_DIR_NAME)
        .join(CONFIG_FILE_NAME)
}

fn read_config_from_path(path: &Path) -> Config {
    let content = std::fs::read_to_string(path).unwrap();
    toml::from_str(&content).unwrap()
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
struct Config {
    #[serde(default)]
    ui: UiConfig,
    #[serde(default)]
    graph: GraphConfig,
    // The user customed keybinds, please ref `assets/default-keybind.toml`
    keybind: Option<KeyBind>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct UiConfig {
    #[serde(default)]
    pub list: UiListConfig,
    #[serde(default)]
    pub detail: UiDetailConfig,
    #[serde(default)]
    pub refs: UiRefsConfig,
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
    #[serde(default = "ui_detail_height_default")]
    pub height: u16,
    #[serde(default = "ui_detail_date_format_default")]
    pub date_format: String,
    #[serde(default = "ui_detail_date_local_default")]
    pub date_local: bool,
}

impl Default for UiDetailConfig {
    fn default() -> Self {
        Self {
            height: ui_detail_height_default(),
            date_format: ui_detail_date_format_default(),
            date_local: ui_detail_date_local_default(),
        }
    }
}

fn ui_detail_height_default() -> u16 {
    DEFAULT_DETAIL_HEIGHT
}

fn ui_detail_date_format_default() -> String {
    DEFAULT_DETAIL_DATE_FORMAT.to_string()
}

fn ui_detail_date_local_default() -> bool {
    DEFAULT_DETAIL_DATE_LOCAL
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UiRefsConfig {
    #[serde(default = "ui_refs_width_default")]
    pub width: u16,
}

impl Default for UiRefsConfig {
    fn default() -> Self {
        Self {
            width: ui_refs_width_default(),
        }
    }
}

fn ui_refs_width_default() -> u16 {
    DEFAULT_REFS_WIDTH
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct GraphConfig {
    #[serde(default)]
    pub color: GraphColorConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GraphColorConfig {
    #[serde(default = "graph_color_branches_default")]
    pub branches: Vec<String>,
    #[serde(default = "graph_color_edge_default")]
    pub edge: String,
    #[serde(default = "graph_color_background_default")]
    pub background: String,
}

impl Default for GraphColorConfig {
    fn default() -> Self {
        Self {
            branches: graph_color_branches_default(),
            edge: graph_color_edge_default(),
            background: graph_color_background_default(),
        }
    }
}

fn graph_color_branches_default() -> Vec<String> {
    DEFAULT_GRAPH_COLOR_BRANCHES
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn graph_color_edge_default() -> String {
    DEFAULT_GRAPH_COLOR_EDGE.into()
}

fn graph_color_background_default() -> String {
    DEFAULT_GRAPH_COLOR_BACKGROUND.into()
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
                    height: 20,
                    date_format: "%Y-%m-%d %H:%M:%S %z".into(),
                    date_local: true,
                },
                refs: UiRefsConfig { width: 26 },
            },
            graph: GraphConfig {
                color: GraphColorConfig {
                    branches: vec![
                        "#E06C76".into(),
                        "#98C379".into(),
                        "#E5C07B".into(),
                        "#61AFEF".into(),
                        "#C678DD".into(),
                        "#56B6C2".into(),
                    ],
                    edge: "#00000000".into(),
                    background: "#00000000".into(),
                },
            },
            keybind: None,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_config_complete_toml() {
        let toml = r##"
            [ui.list]
            subject_min_width = 40
            date_format = "%Y/%m/%d"
            date_width = 20
            date_local = false
            name_width = 30
            [ui.detail]
            height = 30
            date_format = "%Y/%m/%d %H:%M:%S"
            date_local = false
            [ui.refs]
            width = 40
            [graph.color]
            branches = ["#ff0000", "#00ff00", "#0000ff"]
            edge = "#000000"
            background = "#ffffff"
        "##;
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
                    height: 30,
                    date_format: "%Y/%m/%d %H:%M:%S".into(),
                    date_local: false,
                },
                refs: UiRefsConfig { width: 40 },
            },
            graph: GraphConfig {
                color: GraphColorConfig {
                    branches: vec!["#ff0000".into(), "#00ff00".into(), "#0000ff".into()],
                    edge: "#000000".into(),
                    background: "#ffffff".into(),
                },
            },
            keybind: None,
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
                    height: 20,
                    date_format: "%Y-%m-%d %H:%M:%S %z".into(),
                    date_local: true,
                },
                refs: UiRefsConfig { width: 26 },
            },
            graph: GraphConfig {
                color: GraphColorConfig {
                    branches: vec![
                        "#E06C76".into(),
                        "#98C379".into(),
                        "#E5C07B".into(),
                        "#61AFEF".into(),
                        "#C678DD".into(),
                        "#56B6C2".into(),
                    ],
                    edge: "#00000000".into(),
                    background: "#00000000".into(),
                },
            },
            keybind: None,
        };
        assert_eq!(actual, expected);
    }
}
