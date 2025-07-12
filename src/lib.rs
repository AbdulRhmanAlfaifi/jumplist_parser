//! Jumplist Parser crate for analyzing Windows Jumplist files.
//!
//! Supports both:
//! - `automaticDestinations-ms` (DestList + LNKs in CFB)
//! - `customDestinations-ms` (CustomDestinations format)
//!

pub mod appids;
pub mod custom_destinations;
pub mod destlist;
pub mod errors;

use cfb::CompoundFile;
use destlist::DestList;
use errors::JumplistParserError;
use std::{
    collections::HashMap,
    fmt::{self, Display},
    fs::File,
    io::{Cursor, Read},
};

use winparsingtools::traits::Normalize;

use serde::Serialize;

use crate::{appids::APPID_TO_NAME, custom_destinations::CustomDestinations};

/// Type of Jumplist file.
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum JumplistType {
    /// Automatic Jumplist (CFB + DestList + LNKs). File extension: `.automaticDestinations-ms`.
    Automatic,
    /// Custom Jumplist (`.customDestinations-ms`). File extension: `.customDestinations-ms`.
    Custom,
}

impl Display for JumplistType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            JumplistType::Automatic => "automatic",
            JumplistType::Custom => "custom",
        };
        write!(f, "{}", s)
    }
}

/// Wrapper enum to hold parsed Jumplist data.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum JumplistData {
    DestList(DestList),
    CustomDestinations(CustomDestinations),
}

/// Parse & represent a Jumplist file data.
#[derive(Debug, Serialize)]
pub struct JumplistParser {
    pub app_id: Option<String>,
    pub app_name: Option<String>,
    pub r#type: JumplistType,
    pub source_path: Option<String>,
    pub data: JumplistData,
}

impl JumplistParser {
    /// Parse a Jumplist from a buffer (Cursor).
    ///
    /// # Arguments
    /// * `r` - Cursor over the file contents.
    /// * `jumplist_type` - Whether it's automatic or custom format.

    pub fn from_reader(
        r: &mut Cursor<Vec<u8>>,
        jumplist_type: JumplistType,
    ) -> Result<Self, JumplistParserError> {
        match jumplist_type {
            JumplistType::Automatic => {
                let mut parser = match CompoundFile::open(r) {
                    Ok(r) => Ok(r),
                    Err(e) => Err(e),
                }
                .map_err(|_| {
                    JumplistParserError::FileStructure(
                        "Unable to parse the file".to_string(),
                        line!(),
                        file!().to_string(),
                    )
                })?;

                let mut destlist_data: Cursor<Vec<u8>> = Cursor::new(vec![]);
                let entries: Vec<cfb::Entry> = parser.walk().collect();

                for entry in entries.iter() {
                    if entry.name() == "DestList" {
                        if entry.len() > 0 {
                            let dl_data = {
                                let mut stream = parser.open_stream("DestList").unwrap();
                                let mut buffer = Vec::new();
                                stream.read_to_end(&mut buffer).unwrap();
                                buffer
                            };

                            destlist_data = Cursor::new(dl_data);
                        } else {
                            // TODO: Handle empty DestList
                        }
                    }
                }

                let data = match destlist::DestList::from_reader(
                    &mut destlist_data,
                    Some(entries),
                    &mut parser,
                ) {
                    Ok(dlist) => Some(dlist),
                    Err(e) => {
                        eprintln!("ERROR: {}", e);
                        None
                    }
                };

                match data {
                    Some(results) => Ok(Self {
                        app_id: None,
                        app_name: None,
                        source_path: None,
                        r#type: jumplist_type,
                        data: JumplistData::DestList(results),
                    }),
                    None => Err(JumplistParserError::NoDestList(
                        "No entry with the name 'DestList' (Empty JumpList)".to_string(),
                        line!(),
                        file!().to_string(),
                    )),
                }
            }
            JumplistType::Custom => {
                let results = CustomDestinations::from_reader(r)?;
                Ok(Self {
                    app_id: None,
                    app_name: None,
                    source_path: None,
                    r#type: jumplist_type,
                    data: JumplistData::CustomDestinations(results),
                })
            }
        }
    }

    /// Parse a Jumplist from a file on disk.
    ///
    /// Automatically detects the Jumplist type based on the file extension:
    /// - `.automaticDestinations-ms` → `JumplistType::Automatic`
    /// - `.customDestinations-ms` → `JumplistType::Custom`
    ///
    /// # Arguments
    /// * `path` - Path to the Jumplist file.
    ///
    /// # Errors
    /// Returns a [`JumplistParserError`] if the file cannot be read, parsed, or its type is unknown.
    ///
    /// # Example
    /// ```
    /// use jumplist_parser::JumplistParser;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let parsed = JumplistParser::from_path("samples/win11/automaticDestinations/4cb9c5750d51c07f.automaticDestinations-ms")?;
    ///
    ///     println!("App ID: {:?}", parsed.app_id);
    ///     println!("Entries: {:?}", parsed);
    ///     Ok(())
    /// }
    /// ```

    pub fn from_path(path: &str) -> Result<Self, JumplistParserError> {
        let mut file = File::open(path).map_err(|e| {
            JumplistParserError::JumplistParser(
                format!("Can't open the file '{}', ERROR: {}", path, e),
                line!(),
                file!().to_string(),
            )
        })?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| {
            JumplistParserError::JumplistParser(
                format!("Can't read the file '{}', ERROR: {}", path, e),
                line!(),
                file!().to_string(),
            )
        })?;
        let mut cursor = Cursor::new(buffer);

        let mut app_id = String::new();
        let mut app_name = String::new();

        let file_name = std::path::PathBuf::from(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let jumplist_type = match file_name.ends_with(".automaticDestinations-ms") {
            true => JumplistType::Automatic,
            false => match file_name.ends_with(".customDestinations-ms") {
                true => JumplistType::Custom,
                false => {
                    return Err(JumplistParserError::FileType(
                        file_name,
                        line!(),
                        file!().to_string(),
                    ))
                }
            },
        };

        if let Some(stem) = file_name.split('.').next() {
            app_id = stem.to_string();
            if let Some(name) = APPID_TO_NAME.get(stem) {
                app_name = name.to_string();
            }
        }

        let parsed = Self::from_reader(&mut cursor, jumplist_type);
        match parsed {
            Ok(mut parsed) => {
                parsed.app_id = Some(app_id);
                parsed.app_name = Some(app_name);
                parsed.source_path = Some(path.to_string());
                Ok(parsed)
            }
            Err(e) => Err(e),
        }
    }
}

/// Trait to normalize parsed structures into a consistent `key` and `value` format.
pub trait Flaten {
    /// Converts the structure into a list of `key` and `value` maps.
    fn flaten(&self) -> Vec<HashMap<String, String>>;
}

impl Flaten for JumplistParser {
    /// Normalize parsed Jumplist entries to flat `key` and `value` maps.
    ///
    /// Adds a `jumplist_file_path` key for traceability.
    fn flaten(&self) -> Vec<HashMap<String, String>> {
        let mut results: Vec<HashMap<String, String>> = Vec::new();

        match &self.data {
            JumplistData::DestList(data) => {
                for entry in &data.entries {
                    let mut e = entry.normalize();
                    let path = match &self.source_path {
                        Some(p) => p.to_owned(),
                        None => String::new(),
                    };
                    e.insert("jumplist_file_path".to_string(), path);
                    results.push(e);
                }
                results
            }
            JumplistData::CustomDestinations(data) => {
                let data = data.flaten();
                for mut entry in data {
                    let path = match &self.source_path {
                        Some(p) => p.to_owned(),
                        None => String::new(),
                    };
                    entry.insert("jumplist_file_path".to_string(), path);
                    results.push(entry);
                }
                results
            }
        }
    }
}
