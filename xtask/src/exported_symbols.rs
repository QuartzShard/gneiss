//! Rust side structure of the exported_symbols.json. We allow dead code here so that the sections
//! we don't read (but still want for completeness) aren't polluting clippy.
#![allow(dead_code)]

use serde::Deserialize;
use std::{fmt::Display, path::PathBuf};

#[derive(Debug, Deserialize)]
/// Schema of exported_symbols.json
pub struct ExportedSymbols {
    #[serde(rename = "_notes")]
    pub notes: Vec<String>,
    #[serde(deserialize_with = "de_str_u32")]
    pub revision: u32,
    pub version: String,
    pub files: Vec<PathBuf>,
    pub fonts: Vec<FontEntry>,
    pub exports: Vec<Export>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum FontEntry {
    Named(String),
    Versioned {
        name: String,
        #[serde(rename = "addedRevision")]
        added_revision: u32,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Export {
    Function(Function),
    Type(TypeDef),
    Define(Named),
    ForwardStruct(Named),
    Group(Group),
}

/// `define` and `forward_struct`: name-only entries.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Named {
    pub name: String,
    #[serde(default, deserialize_with = "de_str_u32")]
    pub added_revision: u32,
    #[serde(default)]
    pub internal: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeDef {
    pub name: String,
    #[serde(default, deserialize_with = "de_str_u32")]
    pub added_revision: u32,
    #[serde(default)]
    pub internal: bool,
    /// Headers to `#include` after the typedef (e.g. gcolor_definitions.h).
    #[serde(default)]
    pub include_after: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Function {
    pub name: String,
    #[serde(default, deserialize_with = "de_str_u32")]
    pub added_revision: u32,
    #[serde(default)]
    pub internal: bool,
    /// Real C symbol when it differs from `name`; shim/index keys off this.
    pub impl_name: Option<String>,
    pub sort_name: Option<String>,
    #[serde(default)]
    pub deprecated: Flag,
    #[serde(default)]
    pub removed: Flag,
    #[serde(default)]
    pub skip_definition: bool,
    pub stub_definition: Option<String>,
    pub stub_return: Option<String>,
    pub api_exists: Option<bool>,
    #[serde(rename = "_comment")]
    pub comment: Option<String>,
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = &self.name;
        let deprecated = match &self.deprecated {
            Flag::Set(false) => "".to_string(),
            Flag::Set(true) => "[deprecated]".to_string(),
            Flag::Reason(reason) => format!("[deprecated: {reason}"),
        };
        let rev = format!("\n  -> Added in {}", self.added_revision);

        write!(f, "{name}{deprecated}{rev}")
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub name: String,
    #[serde(default, deserialize_with = "de_str_u32")]
    pub added_revision: u32,
    #[serde(default)]
    pub internal: bool,
    #[serde(default)]
    pub deprecated: Flag,
    #[serde(default)]
    pub app_only: bool,
    #[serde(default)]
    pub worker_only: bool,
    pub note: Option<String>,
    pub exports: Vec<Export>,
}

/// `removed`/`deprecated` are either `true` or a string message.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Flag {
    Set(bool),
    Reason(String),
}

impl Default for Flag {
    fn default() -> Self {
        Flag::Set(false)
    }
}

impl Flag {
    pub fn is_set(&self) -> bool {
        !matches!(self, Flag::Set(false))
    }
}

fn de_str_u32<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
    let s = String::deserialize(d)?;
    s.parse().map_err(serde::de::Error::custom)
}
