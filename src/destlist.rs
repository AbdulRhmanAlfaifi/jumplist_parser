//! Parser for `DestList` streams in Windows JumpList automatic destinations files.
//! These contain metadata about recently or frequently accessed files, including
//! a reference to LNK entries stored in the same compound file.

use crate::errors::JumplistParserError;
use byteorder::{LittleEndian, ReadBytesExt};
use lnk_parser::LNKParser;
use serde::Serialize;
use std::{
    collections::HashMap,
    io::{Cursor, Read, Seek, SeekFrom},
};
use winparsingtools::{
    date_time::FileTime,
    structs::Guid,
    traits::Normalize,
    utils::{read_utf16_string, read_utf8_string},
};

use crate::_Normalize;

/// Represents the header of a `DestList` stream.
#[derive(Debug, Serialize)]
pub struct DestListHeader {
    pub version: u32,
    pub number_of_entries: u32,
    pub number_of_pinned_entries: u32,
}

impl DestListHeader {
    /// Parse a `DestListHeader` from a raw byte buffer.
    pub fn from_buffer(buf: &[u8]) -> Result<Self, JumplistParserError> {
        Self::from_reader(&mut Cursor::new(buf))
    }

    /// Parse a `DestListHeader` from a readable and seekable stream.
    pub fn from_reader<R: Read + Seek>(r: &mut R) -> Result<Self, JumplistParserError> {
        let version = r.read_u32::<LittleEndian>().map_err(|_| {
            JumplistParserError::DestListHeader(
                "Can't parse the 'version'".to_string(),
                line!(),
                file!().to_string(),
            )
        })?;
        let number_of_entries = r.read_u32::<LittleEndian>().map_err(|_| {
            JumplistParserError::DestListHeader(
                "Can't parse the 'number_of_entries'".to_string(),
                line!(),
                file!().to_string(),
            )
        })?;
        let number_of_pinned_entries = r.read_u32::<LittleEndian>().map_err(|_| {
            JumplistParserError::DestListHeader(
                "Can't parse the 'number_of_pinned_entries'".to_string(),
                line!(),
                file!().to_string(),
            )
        })?;
        // Ignore unknown bytes
        r.seek(SeekFrom::Current(20)).map_err(|_| {
            JumplistParserError::DestListHeader(
                "Can't seek after the unknow bytes".to_string(),
                line!(),
                file!().to_string(),
            )
        })?;

        Ok(Self {
            version,
            number_of_entries,
            number_of_pinned_entries,
        })
    }
}

/// Represents a single entry in the DestList stream.
#[derive(Debug, Serialize)]
pub struct DestListEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_id: Option<usize>,
    /// GUID of the volume the file resides on.
    pub volume_droid: Guid,
    /// GUID of the file itself.
    pub file_droid: Guid,
    /// Volume birth GUID.
    pub volume_birth_droid: Guid,
    /// File birth GUID.
    pub file_birth_droid: Guid,
    /// Hostname where the file was accessed.
    pub hostname: String,
    /// Entry index number that corresponds with the LNK file with the same number in hex in the same compund file.
    pub entry_number: u32,
    /// Last modification time.
    pub mtime: FileTime,
    /// Indicates whether the entry is pinned.
    pub pined: bool,
    /// UTF-16 path of the file.
    pub path: String,
    /// Parsed LNK entry associated with this entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lnk: Option<LNKParser>,
}

impl DestListEntry {
    /// Parses a `DestListEntry` from a buffer.
    pub fn from_buffer(buf: &[u8], version: u32) -> Result<Self, JumplistParserError> {
        Self::from_reader(&mut Cursor::new(buf), version)
    }

    /// Parses a `DestListEntry` from a readable and seekable stream.
    pub fn from_reader<R: Read + Seek>(
        r: &mut R,
        version: u32,
    ) -> Result<Self, JumplistParserError> {
        // Ignore unknown bytes
        r.seek(SeekFrom::Current(8)).map_err(|_| {
            JumplistParserError::DestListEntry(
                "Can't seek after unknown 8 bytes".to_string(),
                line!(),
                file!().to_string(),
            )
        })?;
        let volume_droid = Guid::from_reader(r).map_err(|_| {
            JumplistParserError::DestListEntry(
                "Can't parse the 'volume_droid'".to_string(),
                line!(),
                file!().to_string(),
            )
        })?;
        let file_droid = Guid::from_reader(r).map_err(|_| {
            JumplistParserError::DestListEntry(
                "Can't parse the 'file_droid'".to_string(),
                line!(),
                file!().to_string(),
            )
        })?;
        let volume_birth_droid = Guid::from_reader(r).map_err(|_| {
            JumplistParserError::DestListEntry(
                "Can't parse the 'volume_birth_droid'".to_string(),
                line!(),
                file!().to_string(),
            )
        })?;
        let file_birth_droid = Guid::from_reader(r).map_err(|_| {
            JumplistParserError::DestListEntry(
                "Can't parse the 'file_birth_droid'".to_string(),
                line!(),
                file!().to_string(),
            )
        })?;
        // The hostname is 16 bytes
        let hostname = read_utf8_string(r, Some(16)).map_err(|_| {
            JumplistParserError::DestListEntry(
                "Can't parse the 'hostname'".to_string(),
                line!(),
                file!().to_string(),
            )
        })?;
        let entry_number = r.read_u32::<LittleEndian>().map_err(|_| {
            JumplistParserError::DestListEntry(
                "Can't parse the 'entry_number'".to_string(),
                line!(),
                file!().to_string(),
            )
        })?;
        // Ignore unknown bytes
        r.seek(SeekFrom::Current(8)).map_err(|_| {
            JumplistParserError::DestListEntry(
                "Can't seek after unknown 8 bytes".to_string(),
                line!(),
                file!().to_string(),
            )
        })?;
        let mtime = FileTime::new(r.read_u64::<LittleEndian>().map_err(|_| {
            JumplistParserError::DestListEntry(
                "Can't parse the 'mtime'".to_string(),
                line!(),
                file!().to_string(),
            )
        })?);
        // Ignore pinned items order and only return true if the item is pinned
        let pined = match r.read_u32::<LittleEndian>().map_err(|_| {
            JumplistParserError::DestListEntry(
                "Can't parse the 'pined'".to_string(),
                line!(),
                file!().to_string(),
            )
        })? {
            0xffffffff => false,
            _ => true,
        };
        if version > 1 {
            // Ignore unknown bytes
            r.seek(SeekFrom::Current(16)).map_err(|_| {
                JumplistParserError::DestListEntry(
                    "Can't seek after unknown 16 bytes".to_string(),
                    line!(),
                    file!().to_string(),
                )
            })?;
        }
        let path_size = r.read_u16::<LittleEndian>().map_err(|_| {
            JumplistParserError::DestListEntry(
                "Can't parse the 'path_size'".to_string(),
                line!(),
                file!().to_string(),
            )
        })?;
        let path = read_utf16_string(r, Some(path_size as usize)).map_err(|_| {
            JumplistParserError::DestListEntry(
                "Can't parse the 'path'".to_string(),
                line!(),
                file!().to_string(),
            )
        })?;

        if version > 1 {
            // Ignore unknown bytes
            r.seek(SeekFrom::Current(4)).map_err(|_| {
                JumplistParserError::DestListEntry(
                    "Can't seek after unknown 4 bytes".to_string(),
                    line!(),
                    file!().to_string(),
                )
            })?;
        }

        Ok(Self {
            volume_droid,
            file_droid,
            volume_birth_droid,
            file_birth_droid,
            hostname,
            entry_number,
            mtime,
            pined,
            path,
            lnk: None,
            entry_id: None,
        })
    }

    /// Tries to parse and attach an LNK entry to this DestList entry.
    fn process_lnk(&mut self, lnk: &[u8]) {
        self.lnk = match LNKParser::from_buffer(&lnk) {
            Ok(a) => Some(a),
            Err(_) => None,
        }
    }
}

/// Represents a parsed `DestList` stream with optional LNK parsing.
#[derive(Debug, Serialize)]
pub struct DestList {
    pub header: DestListHeader,
    pub entries: Vec<DestListEntry>,
}

impl DestList {
    /// Parses a DestList stream and associated LNK entries from a CFB compound file.
    pub fn from_reader<R: Read + Seek>(
        r: &mut R,
        lnks: Option<Vec<cfb::Entry>>,
        parser: &mut cfb::CompoundFile<&mut R>,
    ) -> Result<Self, JumplistParserError> {

        let dlist_size = match &lnks {
            Some(entries) => {
                let mut size = 0;
                for entry in entries.iter() {
                    if entry.name() == "DestList" {
                        size = entry.len();
                    }
                }
                size
            }
            None => 0
        };
        let header = match dlist_size { 
            0 => {
                Ok(DestListHeader {
                    version: 0,
                    number_of_entries: 0,
                    number_of_pinned_entries: 0,
                })
            }
            _ => DestListHeader::from_reader(r)
        }?;
        let mut entries: Vec<DestListEntry> = vec![];

        loop {
            match &lnks {
                Some(ls) => match DestListEntry::from_reader(r, header.version) {
                    Ok(mut entry) => {
                        for lnk in ls {
                            if format!("{:x?}", entry.entry_number) == lnk.name() {
                                let lnk_data = {
                                    let stream = parser.open_stream(lnk.path()).map_err(|e| {
                                        JumplistParserError::LnkEntry(format!("Error reading LNK file '{}', CFB_ERROR: {}", lnk.name(), e), line!(), file!().to_string())
                                    });

                                    match stream {
                                        Ok(mut s) => {
                                            let mut buffer = Vec::new();
                                            s.read_to_end(&mut buffer).unwrap();
                                            buffer

                                        }
                                        Err(e) => {
                                            eprintln!("{}", e);
                                            continue;
                                        } 
                                    }
                                };

                                entry.process_lnk(&lnk_data)
                            }
                        }
                        entries.push(entry);
                    }
                    Err(_) => break,
                },
                None => match DestListEntry::from_reader(r, header.version) {
                    Ok(entry) => entries.push(entry),
                    Err(_) => break,
                },
            }
        }
        entries.sort_by(|a, b| b.entry_number.cmp(&a.entry_number));

        Ok(Self { header, entries })
    }
}

impl Normalize for DestListEntry {
    /// Normalizes the internal LNK entry (if present) and returns selected fields.
    fn normalize(&self) -> HashMap<String, String> {
        let results: HashMap<String, String> = HashMap::new();
        match &self.lnk {
            Some(l) => {
                let mut lnk_normalized = l.normalize();
                let name_string = match l.get_name_string() {
                    Some(s) => s.to_string(),
                    None => String::from("")
                };

                let command_line_arguments = match l.get_command_line_arguments() {
                    Some(s) => s.to_string(),
                    None => String::from("")
                };
                lnk_normalized.insert("name_string".to_string(), name_string);
                lnk_normalized.insert("command_line_arguments".to_string(), command_line_arguments);
                lnk_normalized
            },
            None => results,
        }
    }
}

impl _Normalize for DestList {
    /// Normalizes all entries and returns a list of `key` and `value` maps.
    fn normalize(&self) -> Vec<HashMap<String, String>> {
        let mut results: Vec<HashMap<String, String>> = Vec::new();
        for entry in &self.entries {
            results.push(entry.normalize());
        }
        results
    }
}
