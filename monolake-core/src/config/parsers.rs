use anyhow::{bail, Result};
use bytes::Bytes;
use serde::de::DeserializeOwned;

pub(crate) fn parse<T: DeserializeOwned>(extension: String, raw: &Bytes) -> Result<T> {
    let parser_type: ParserType = extension.clone().into();
    match parser_type {
        ParserType::Json(parser) => parser.parse(&raw[..raw.len()]),
        ParserType::Toml(parser) => parser.parse(&raw[..raw.len()]),
        ParserType::Unsupported => {
            bail!("No parser available for config format {}", extension)
        }
    }
}

enum ParserType {
    Json(JsonParser),
    Toml(TomlParser),
    Unsupported,
}

impl Into<ParserType> for String {
    fn into(self) -> ParserType {
        match self.to_ascii_lowercase().as_str() {
            "json" => ParserType::Json(Default::default()),
            "toml" => ParserType::Toml(Default::default()),
            _ => ParserType::Unsupported,
        }
    }
}

trait Parser {
    fn parse<T: DeserializeOwned>(&self, raw: &[u8]) -> Result<T>;
}

#[derive(Default)]
struct JsonParser;

impl Parser for JsonParser {
    fn parse<'a, T: DeserializeOwned>(&self, raw: &[u8]) -> Result<T> {
        match serde_json::from_slice::<T>(raw) {
            Ok(t) => Ok(t),
            Err(e) => bail!(e),
        }
    }
}

#[derive(Default)]
struct TomlParser;

impl Parser for TomlParser {
    fn parse<T: DeserializeOwned>(&self, raw: &[u8]) -> Result<T> {
        let content = String::from_utf8(raw.to_vec())?;
        match toml::from_str(&content) {
            Ok(t) => Ok(t),
            Err(e) => bail!(e),
        }
    }
}
