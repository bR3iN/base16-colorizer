use std::convert::AsRef;
use std::fs::File;
use std::path::Path;

use serde::Deserialize;

use crate::color::Color;
use crate::utils::Context;
use crate::Result;

#[derive(Debug, Deserialize)]
#[serde(from = "YamlScheme")]
pub struct ColorScheme {
    pub author: String,
    pub name: String,
    pub colors: [Color; 16],
}

impl ColorScheme {
    pub fn try_from_yaml(path: impl AsRef<Path>) -> Result<ColorScheme> {
        let handle = File::open(&path)
            .add_context(|| format!("Failed to open {}", path.as_ref().display()))?;
        Ok(serde_yaml::from_reader(handle)
            .add_context(|| format!("In {}", path.as_ref().display()))?)
    }
}

/// Helper struct to deserialize `ColorScheme`s from YAML files.
#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct YamlScheme {
    pub author: String,
    pub scheme: String,
    pub base00: Color,
    pub base01: Color,
    pub base02: Color,
    pub base03: Color,
    pub base04: Color,
    pub base05: Color,
    pub base06: Color,
    pub base07: Color,
    pub base08: Color,
    pub base09: Color,
    pub base0A: Color,
    pub base0B: Color,
    pub base0C: Color,
    pub base0D: Color,
    pub base0E: Color,
    pub base0F: Color,
}

impl From<YamlScheme> for ColorScheme {
    fn from(yaml: YamlScheme) -> ColorScheme {
        ColorScheme {
            author: yaml.author,
            name: yaml.scheme,
            colors: [
                yaml.base00,
                yaml.base01,
                yaml.base02,
                yaml.base03,
                yaml.base04,
                yaml.base05,
                yaml.base06,
                yaml.base07,
                yaml.base08,
                yaml.base09,
                yaml.base0A,
                yaml.base0B,
                yaml.base0C,
                yaml.base0D,
                yaml.base0E,
                yaml.base0F,
            ],
        }
    }
}
