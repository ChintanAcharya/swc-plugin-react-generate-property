use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PluginOptions {
    #[serde(default = "default_custom_property")]
    custom_property: String,

    #[serde(default = "default_custom_separator")]
    custom_separator: String,

    slash_char: String,

    #[serde(default = "default_dir_level")]
    dir_level: usize,

    add_module_class_names: bool,
    prefix: String,
    ignore_tree_depth: bool,
    ignore_node_names: bool,
    first_child_only: bool,
    omit_file_name: bool,
    r#match: Option<String>,
}

fn default_custom_property() -> String {
    "data-id".to_string()
}

fn default_custom_separator() -> String {
    "_".to_string()
}

fn default_dir_level() -> usize {
    1
}
