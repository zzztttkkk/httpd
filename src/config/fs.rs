use serde::Deserialize;

#[derive(Deserialize, Clone, Default)]
pub struct ConfigFS {
    #[serde(default)]
    root: String,

    #[serde(default)]
    prefix: String,

    #[serde(default)]
    disable_index: bool,

    #[serde(default)]
    index_format: String, // html or json

    #[serde(default)]
    allow_upload: String,
}
