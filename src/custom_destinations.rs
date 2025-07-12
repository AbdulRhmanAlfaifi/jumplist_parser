//! Parser for CustomDestinations Jumplist files used by Windows.
//! These files are part of the Jump List feature and typically
//! contain LNK entries grouped under different categories.

use byteorder::{LittleEndian, ReadBytesExt};
use lnk_parser::LNKParser;
use serde::{Serialize, Serializer};
use winparsingtools::structs::Guid;
use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::Flaten;
use crate::errors::JumplistParserError;
use winparsingtools::{traits::Normalize, utils::read_utf16_string};

/// Category types used in CustomDestinations.
/// - `Custom`: User-defined or application-defined category.
/// - `Known`: Special categories like "Recent" or "Frequent".
/// - `Task`: Represents shortcut tasks like creating new project.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CatagoryType {
    Custom = 0x00,
    Known = 0x01,
    Task = 0x02,
}

/// Represents the file header of a `.customDestinations-ms` file.
#[derive(Debug, Serialize)]
pub struct CustomDestinationsHeader {
    /// File format version
    pub version: u32,
    /// Number of categories
    pub num_of_cat: u32,
    /// Unknown field, seen as 0x0 always this might be a reserved.
    #[serde(skip_serializing)]
    pub unkonwn: u32,
}

impl CustomDestinationsHeader {
    /// Parses the header from a file path.
    pub fn from_path(path: &str) -> Result<Self, JumplistParserError> {
        let mut file = std::fs::File::open(path).map_err(|e| {
            JumplistParserError::General(e.to_string(), line!(), file!().to_string())
        })?;
        Self::from_reader(&mut file)
    }
    /// Parses the header from a given reader.
    pub fn from_reader<R: Read + Seek>(reader: &mut R) -> Result<Self, JumplistParserError> {
        let version = reader.read_u32::<LittleEndian>().map_err(|e| {
            JumplistParserError::FileStructure(e.to_string(), line!(), file!().to_string())
        })?;
        let num_of_cat = reader.read_u32::<LittleEndian>().map_err(|e| {
            JumplistParserError::FileStructure(e.to_string(), line!(), file!().to_string())
        })?;
        let unkonwn = reader.read_u32::<LittleEndian>().map_err(|e| {
            JumplistParserError::FileStructure(e.to_string(), line!(), file!().to_string())
        })?;

        Ok(CustomDestinationsHeader {
            version,
            num_of_cat,
            unkonwn,
        })
    }
}

/// IDs of categories. either `Frequent` or `Recent`.
#[derive(Debug)]
#[repr(i32)]
pub enum CategoryID {
    Frequent = 0x01,
    Recent = 0x02,
    None = -1,
    /// Unknown or unrecognized category ID.
    Unknown(i32),
}

impl Serialize for CategoryID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CategoryID::Frequent => serializer.serialize_str("frequent"),
            CategoryID::Recent => serializer.serialize_str("recent"),
            CategoryID::None => serializer.serialize_str("none"),
            CategoryID::Unknown(val) => serializer.serialize_str(&format!("{:04X}", val)),
        }
    }
}

/// Represents a category inside a CustomDestinations file.
/// A category groups one or more LNK entries or Shellitems.
#[derive(Debug, Serialize)]
pub struct Catagory {
    /// Type of the category (`Custom`, `Known` or `Task`).
    pub r#type: CatagoryType,
    /// Name of the category (only for `Custom`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Number of LNK entries or Shellitems.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_of_entries: Option<u32>,
    /// Known category ID (used only when `type` is `Known`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<CategoryID>,
    /// Parsed LNK entries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<LNKParser>>,
}

/// Represents the entire parsed CustomDestinations jumplist file.
///
/// # Example
/// ```rust
/// use jumplist_parser::custom_destinations::CustomDestinations;
///
/// fn main() {
///     let cd = CustomDestinations::from_path("samples/win11/CustomDestinations/1ced32d74a95c7bc.customDestinations-ms").unwrap();
///     println!("{:#?}", cd);
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct CustomDestinations {
    /// File header with metadata.
    pub header: CustomDestinationsHeader,
    /// All parsed categories and their LNK entries.
    pub entries: Vec<Catagory>,
}

impl CustomDestinations {
    /// Parse a CustomDestinations file from a path on disk.
    pub fn from_path(path: &str) -> Result<Self, JumplistParserError> {
        let mut file = std::fs::File::open(path).map_err(|e| {
            JumplistParserError::FileStructure(e.to_string(), line!(), file!().to_string())
        })?;
        Self::from_reader(&mut file)
    }

    /// Parse a CustomDestinations file from a reader.
    pub fn from_reader<R: Read + Seek>(reader: &mut R) -> Result<Self, JumplistParserError> {
        let header = CustomDestinationsHeader::from_reader(reader)?;
        let mut categories = Vec::new();

        for _ in 0..header.num_of_cat {
            let r#type = reader.read_u32::<LittleEndian>().map_err(|e| {
                JumplistParserError::FileStructure(e.to_string(), line!(), file!().to_string())
            })?;
            let r#type = match r#type {
                0x00 => CatagoryType::Custom,
                0x01 => CatagoryType::Known,
                0x02 => CatagoryType::Task,
                x => {
                    return Err(JumplistParserError::FileStructure(
                        format!("CatagoryType unknown '{}'", x),
                        line!(),
                        file!().to_string(),
                    ))
                }
            };

            if r#type == CatagoryType::Custom {
                let name_len = reader.read_u16::<LittleEndian>().ok().unwrap();
                let name = read_utf16_string(reader, Some(name_len as usize)).ok();
                let num_of_entries = reader.read_u32::<LittleEndian>().ok();

                let mut entries = vec![];
                for _ in 0..num_of_entries.unwrap() {
                    // Ignore the Classs ID. From my testing this is always a LNK strcuture, however it should be checked if it is '00021401-0000-0000-c000-000000000046'
                    // Then it is a LNK file, otherwise it is a shellitem.
                    // reader.seek(SeekFrom::Current(16)).map_err(|e| {
                    //     JumplistParserError::FileStructure(
                    //         e.to_string(),
                    //         line!(),
                    //         file!().to_string(),
                    //     )
                    // })?;

                    let mut guid_data = [0;16];
                    reader.read_exact(&mut guid_data).map_err(|e| {
                        JumplistParserError::FileStructure(e.to_string(), line!(), file!().to_string())
                    })?;

                    let mut r = Cursor::new(guid_data);
                    let guid = Guid::from_reader(&mut r).map_err(|e| {
                        JumplistParserError::FileStructure(e.to_string(), line!(), file!().to_string())
                    })?;
                    
                    // Check if the GUID is the expected one for LNK entries
                    if guid.to_string() != "00021401-0000-0000-C000-000000000046" {
                        return Err(JumplistParserError::FileStructure(
                            format!("Custom Category with unknown entry GUID '{}'", guid),
                            line!(),
                            file!().to_string(),
                        ));
                    }

                    let lnk_entry = LNKParser::from_reader(reader).map_err(|e| {
                        JumplistParserError::LnkEntry(e.to_string(), line!(), file!().to_string())
                    })?;
                    entries.push(lnk_entry);
                }

                categories.push(Catagory {
                    r#type,
                    name,
                    num_of_entries,
                    entries: Some(entries),
                    id: None,
                });
            } else if r#type == CatagoryType::Known {
                let id = reader.read_i32::<LittleEndian>().map_err(|e| {
                    JumplistParserError::FileStructure(e.to_string(), line!(), file!().to_string())
                })?;

                let id = match id {
                    1 => CategoryID::Frequent,
                    2 => CategoryID::Recent,
                    -1 => CategoryID::None,
                    x => CategoryID::Unknown(x),
                };

                categories.push(Catagory {
                    r#type,
                    name: None,
                    num_of_entries: None,
                    id: Some(id),
                    entries: None,
                });
            } else if r#type == CatagoryType::Task {
                let num_of_entries = reader.read_u32::<LittleEndian>().ok();

                let mut entries = vec![];
                for _ in 0..num_of_entries.unwrap() {
                    // Ignore the Classs ID. From my testing this is always a LNK strcuture, however it should be checked if it is '00021401-0000-0000-c000-000000000046'
                    // Then it is a LNK file, otherwise it is a shellitem.
                    // reader.seek(SeekFrom::Current(16)).map_err(|e| {
                    //     JumplistParserError::FileStructure(
                    //         e.to_string(),
                    //         line!(),
                    //         file!().to_string(),
                    //     )
                    // })?;

                    let mut guid_data = [0;16];
                    reader.read_exact(&mut guid_data).map_err(|e| {
                        JumplistParserError::FileStructure(e.to_string(), line!(), file!().to_string())
                    })?;

                    let mut r = Cursor::new(guid_data);
                    let guid = Guid::from_reader(&mut r).map_err(|e| {
                        JumplistParserError::FileStructure(e.to_string(), line!(), file!().to_string())
                    })?;

                    // Check if the GUID is the expected one for LNK entries
                    if guid.to_string() != "00021401-0000-0000-C000-000000000046" {
                        return Err(JumplistParserError::FileStructure(
                            format!("Task Category with unknown entry GUID '{}'", guid),
                            line!(),
                            file!().to_string(),
                        ));
                    }

                    let lnk_entry = LNKParser::from_reader(reader).map_err(|e| {
                        JumplistParserError::LnkEntry(e.to_string(), line!(), file!().to_string())
                    })?;
                    entries.push(lnk_entry);
                }
                categories.push(Catagory {
                    r#type,
                    name: None,
                    num_of_entries,
                    entries: Some(entries),
                    id: None,
                });
            }

            // skip footer
            reader.seek(SeekFrom::Current(4)).map_err(|e| {
                JumplistParserError::FileStructure(e.to_string(), line!(), file!().to_string())
            })?;
        }

        Ok(CustomDestinations {
            header,
            entries: categories,
        })
    }
}

impl Flaten for CustomDestinations {
    /// Normalizes all LNK entries within the CustomDestinations file
    /// into a vector of `key` and `value` maps by exteracting the most important fields.
    ///
    /// Fields like `name_string` and `command_line_arguments` are extracted
    /// to provide meaningful descriptions of the LNK contents.
    fn flaten(&self) -> Vec<HashMap<String, String>> {
        let mut results: Vec<HashMap<String, String>> = Vec::new();
        for entry in &self.entries {
            if let Some(lnks) = &entry.entries {
                for lnk in lnks {
                    let mut lnk_normalized = lnk.normalize();
                    let name_string = match lnk.get_name_string() {
                        Some(s) => s.to_string(),
                        None => String::from(""),
                    };

                    let command_line_arguments = match lnk.get_command_line_arguments() {
                        Some(s) => s.to_string(),
                        None => String::from(""),
                    };
                    lnk_normalized.insert("name_string".to_string(), name_string);
                    lnk_normalized
                        .insert("command_line_arguments".to_string(), command_line_arguments);

                    results.push(lnk_normalized);
                }
            }
        }
        results
    }
}
