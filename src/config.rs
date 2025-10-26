use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use smart_default::SmartDefault;
use umbra::optional;

use crate::{
    color::{ColorTheme, OptionalColorTheme},
    keybind::KeyBind,
    CommitOrderType, GraphWidthType, ImageProtocolType, Result,
};

const XDG_CONFIG_HOME_ENV_NAME: &str = "XDG_CONFIG_HOME";
const DEFAULT_CONFIG_DIR: &str = ".config";
const APP_DIR_NAME: &str = "serie";
const CONFIG_FILE_NAME: &str = "config.toml";
const CONFIG_FILE_ENV_NAME: &str = "SERIE_CONFIG_FILE";

pub fn load() -> Result<(
    CoreConfig,
    UiConfig,
    GraphConfig,
    ColorTheme,
    Option<KeyBind>,
)> {
    let config = match config_file_path_from_env() {
        Some(user_path) => {
            if !user_path.exists() {
                let msg = format!(
                    "Config file specified by ${CONFIG_FILE_ENV_NAME} environment variable not found: {}",
                    user_path.display()
                );
                return Err(msg.into());
            }
            read_config_from_path(&user_path)
        }
        None => {
            if let Some(default_path) = config_file_path() {
                if default_path.exists() {
                    read_config_from_path(&default_path)
                } else {
                    Ok(Config::default())
                }
            } else {
                Ok(Config::default())
            }
        }
    }?;
    Ok((
        config.core,
        config.ui,
        config.graph,
        config.color,
        config.keybind,
    ))
}

fn config_file_path_from_env() -> Option<PathBuf> {
    env::var(CONFIG_FILE_ENV_NAME).ok().map(PathBuf::from)
}

fn config_file_path() -> Option<PathBuf> {
    env::var(XDG_CONFIG_HOME_ENV_NAME)
        .ok()
        .map(PathBuf::from)
        .or_else(|| env::home_dir().map(|home| home.join(DEFAULT_CONFIG_DIR)))
        .map(|config_dir| config_dir.join(APP_DIR_NAME).join(CONFIG_FILE_NAME))
}

fn read_config_from_path(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)?;
    let config: OptionalConfig = toml::from_str(&content)?;
    Ok(config.into())
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Config {
    #[nested]
    core: CoreConfig,
    #[nested]
    ui: UiConfig,
    #[nested]
    graph: GraphConfig,
    #[nested]
    color: ColorTheme,
    // The user customed keybinds, please ref `assets/default-keybind.toml`
    keybind: Option<KeyBind>,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CoreConfig {
    #[nested]
    pub option: CoreOptionConfig,
    #[nested]
    pub search: CoreSearchConfig,
    #[nested]
    pub user_command: CoreUserCommandConfig,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, PartialEq, Eq, SmartDefault)]
pub struct CoreOptionConfig {
    pub protocol: Option<ImageProtocolType>,
    pub order: Option<CommitOrderType>,
    pub graph_width: Option<GraphWidthType>,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, PartialEq, Eq, SmartDefault)]
pub struct CoreSearchConfig {
    #[default = false]
    pub ignore_case: bool,
    #[default = false]
    pub fuzzy: bool,
}

#[optional]
#[derive(Debug, Clone, PartialEq, Eq, SmartDefault)]
pub struct CoreUserCommandConfig {
    #[default(HashMap::from([("1".into(), UserCommand {
        name: "git diff".into(),
        commands: vec![
            "git".into(),
            "--no-pager".into(),
            "diff".into(),
            "--color=always".into(),
            "{{first_parent_hash}}".into(),
            "{{target_hash}}".into(),
        ],
    })]))]
    pub commands: HashMap<String, UserCommand>,
    #[default = 4]
    pub tab_width: u16,
}

impl<'de> Deserialize<'de> for OptionalCoreUserCommandConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, MapAccess, Visitor};
        use std::fmt;

        struct OptionalCoreUserCommandConfigVisitor;

        impl<'de> Visitor<'de> for OptionalCoreUserCommandConfigVisitor {
            type Value = OptionalCoreUserCommandConfig;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a user command configuration")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut commands = HashMap::new();
                let mut tab_width = None;

                while let Some(key) = map.next_key::<String>()? {
                    if let Some(suffix) = key.strip_prefix("commands_") {
                        let command_key = suffix.to_string();
                        if command_key.is_empty() {
                            return Err(V::Error::custom(
                                "command key cannot be empty, like `commands_`",
                            ));
                        }
                        let command_value: UserCommand = map.next_value()?;
                        commands.insert(command_key, command_value);
                    } else if key == "tab_width" {
                        tab_width = Some(map.next_value()?);
                    } else if key == "commands" {
                        return Err(V::Error::custom(
                            "invalid key `commands`, use `commands_n` format instead",
                        ));
                    } else {
                        let _: serde::de::IgnoredAny = map.next_value()?;
                    }
                }

                let commands = if commands.is_empty() {
                    None
                } else {
                    Some(commands)
                };

                Ok(OptionalCoreUserCommandConfig {
                    commands,
                    tab_width,
                })
            }
        }

        deserializer.deserialize_map(OptionalCoreUserCommandConfigVisitor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UserCommand {
    pub name: String,
    pub commands: Vec<String>,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct UiConfig {
    #[nested]
    pub common: UiCommonConfig,
    #[nested]
    pub list: UiListConfig,
    #[nested]
    pub detail: UiDetailConfig,
    #[nested]
    pub user_command: UiUserCommandConfig,
    #[nested]
    pub refs: UiRefsConfig,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, PartialEq, Eq, SmartDefault)]
pub struct UiCommonConfig {
    #[default(CursorType::Native)]
    pub cursor_type: CursorType,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum CursorType {
    Native,
    Virtual(String),
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, PartialEq, Eq, SmartDefault)]
pub struct UiListConfig {
    #[default = 20]
    pub subject_min_width: u16,
    #[default = "%Y-%m-%d"]
    pub date_format: String,
    #[default = 10]
    pub date_width: u16,
    #[default = true]
    pub date_local: bool,
    #[default = 20]
    pub name_width: u16,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, PartialEq, Eq, SmartDefault)]
pub struct UiDetailConfig {
    #[default = 20]
    pub height: u16,
    #[default = "%Y-%m-%d %H:%M:%S %z"]
    pub date_format: String,
    #[default = true]
    pub date_local: bool,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, PartialEq, Eq, SmartDefault)]
pub struct UiUserCommandConfig {
    #[default = 20]
    pub height: u16,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, PartialEq, Eq, SmartDefault)]
pub struct UiRefsConfig {
    #[default = 26]
    pub width: u16,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GraphConfig {
    #[nested]
    pub color: GraphColorConfig,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, PartialEq, Eq, SmartDefault)]
pub struct GraphColorConfig {
    #[default(vec![
        "#E06C76".into(),
        "#98C379".into(),
        "#E5C07B".into(),
        "#61AFEF".into(),
        "#C678DD".into(),
        "#56B6C2".into(),
    ])]
    pub branches: Vec<String>,
    #[default = "#00000000"]
    pub edge: String,
    #[default = "#00000000"]
    pub background: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let actual = Config::default();
        let expected = Config {
            core: CoreConfig {
                option: CoreOptionConfig {
                    protocol: None,
                    order: None,
                    graph_width: None,
                },
                search: CoreSearchConfig {
                    ignore_case: false,
                    fuzzy: false,
                },
                user_command: CoreUserCommandConfig {
                    commands: HashMap::from([(
                        "1".into(),
                        UserCommand {
                            name: "git diff".into(),
                            commands: vec![
                                "git".into(),
                                "--no-pager".into(),
                                "diff".into(),
                                "--color=always".into(),
                                "{{first_parent_hash}}".into(),
                                "{{target_hash}}".into(),
                            ],
                        },
                    )]),
                    tab_width: 4,
                },
            },
            ui: UiConfig {
                common: UiCommonConfig {
                    cursor_type: CursorType::Native,
                },
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
                user_command: UiUserCommandConfig { height: 20 },
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
            color: ColorTheme::default(),
            keybind: None,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_config_complete_toml() {
        let toml = r##"
            [core.option]
            protocol = "kitty"
            order = "topo"
            graph_width = "single"
            [core.search]
            ignore_case = true
            fuzzy = true
            [core.user_command]
            commands_1 = { name = "git diff no color", commands = ["git", "diff", "{{first_parent_hash}}", "{{target_hash}}"] }
            commands_2 = { name = "echo hello", commands = ["echo", "hello"] }
            commands_10 = { name = "echo world", commands = ["echo", "world"] }
            tab_width = 2
            [ui.common]
            cursor_type = { Virtual = "|" }
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
            [ui.user_command]
            height = 30
            [ui.refs]
            width = 40
            [graph.color]
            branches = ["#ff0000", "#00ff00", "#0000ff"]
            edge = "#000000"
            background = "#ffffff"
        "##;
        let actual: Config = toml::from_str::<OptionalConfig>(toml).unwrap().into();
        let expected = Config {
            core: CoreConfig {
                option: CoreOptionConfig {
                    protocol: Some(ImageProtocolType::Kitty),
                    order: Some(CommitOrderType::Topo),
                    graph_width: Some(GraphWidthType::Single),
                },
                search: CoreSearchConfig {
                    ignore_case: true,
                    fuzzy: true,
                },
                user_command: CoreUserCommandConfig {
                    commands: HashMap::from([
                        (
                            "1".into(),
                            UserCommand {
                                name: "git diff no color".into(),
                                commands: vec![
                                    "git".into(),
                                    "diff".into(),
                                    "{{first_parent_hash}}".into(),
                                    "{{target_hash}}".into(),
                                ],
                            },
                        ),
                        (
                            "2".into(),
                            UserCommand {
                                name: "echo hello".into(),
                                commands: vec!["echo".into(), "hello".into()],
                            },
                        ),
                        (
                            "10".into(),
                            UserCommand {
                                name: "echo world".into(),
                                commands: vec!["echo".into(), "world".into()],
                            },
                        ),
                    ]),
                    tab_width: 2,
                },
            },
            ui: UiConfig {
                common: UiCommonConfig {
                    cursor_type: CursorType::Virtual("|".into()),
                },
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
                user_command: UiUserCommandConfig { height: 30 },
                refs: UiRefsConfig { width: 40 },
            },
            graph: GraphConfig {
                color: GraphColorConfig {
                    branches: vec!["#ff0000".into(), "#00ff00".into(), "#0000ff".into()],
                    edge: "#000000".into(),
                    background: "#ffffff".into(),
                },
            },
            color: ColorTheme::default(),
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
        let actual: Config = toml::from_str::<OptionalConfig>(toml).unwrap().into();
        let expected = Config {
            core: CoreConfig {
                option: CoreOptionConfig {
                    protocol: None,
                    order: None,
                    graph_width: None,
                },
                search: CoreSearchConfig {
                    ignore_case: false,
                    fuzzy: false,
                },
                user_command: CoreUserCommandConfig {
                    commands: HashMap::from([(
                        "1".into(),
                        UserCommand {
                            name: "git diff".into(),
                            commands: vec![
                                "git".into(),
                                "--no-pager".into(),
                                "diff".into(),
                                "--color=always".into(),
                                "{{first_parent_hash}}".into(),
                                "{{target_hash}}".into(),
                            ],
                        },
                    )]),
                    tab_width: 4,
                },
            },
            ui: UiConfig {
                common: UiCommonConfig {
                    cursor_type: CursorType::Native,
                },
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
                user_command: UiUserCommandConfig { height: 20 },
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
            color: ColorTheme::default(),
            keybind: None,
        };
        assert_eq!(actual, expected);
    }
}
