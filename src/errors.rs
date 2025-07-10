//! Error types for the Jumplist parser.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum JumplistParserError {
    #[error("Error parsing 'DestList' struct on line '{2}:{1}'. ERROR: '{0}'")]
    DestList(String, u32, String),
    #[error("Error parsing 'DestListHeader' struct on line '{2}:{1}'. ERROR: '{0}'")]
    DestListHeader(String, u32, String),
    #[error("Error parsing 'DestListEntry' struct on line '{2}:{1}'. ERROR: '{0}'")]
    DestListEntry(String, u32, String),
    #[error("Error parsing 'LNK' struct on line '{2}:{1}'. ERROR: '{0}'")]
    LnkEntry(String, u32, String),
    #[error("Error in 'JumplistParser' on line '{2}:{1}'. ERROR: '{0}'")]
    JumplistParser(String, u32, String),
    #[error("Error in 'FileStructure' on line '{2}:{1}'. ERROR: '{0}'")]
    FileStructure(String, u32, String),
    #[error("General error on line '{2}:{1}'. ERROR: '{0}'")]
    General(String, u32, String),
    #[error("Empty JumpList (No DestList) '{2}:{1}'. ERROR: '{0}'")]
    NoDestList(String, u32, String),
    #[error("Unable to indentify Jumplist type (doesn't end with '.automaticDestinations-ms' or '.customDestinations-ms') '{2}:{1}'. Filename: '{0}'")]
    FileType(String, u32, String),
}
