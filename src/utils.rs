use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

//path for storing backup metadata
pub const METADATA_DIR: &str = ".linux_backup_metadata";

#[derive(Clone, Debug)]
pub enum BackupType {
    Full,
    Incremental,
    Differential,
}

#[derive(Serialize, Deserialize, Default)]
pub struct BackupMetadata {
    pub last_backup_time: Option<u64>,
    pub original_backup_time: Option<u64>,
    pub backup_history: HashMap<String, u64>, //path -> timestamp
}

pub fn load_backup_metadata(metadata_dir: &Path) -> Result<BackupMetadata, Box<dyn std::error::Error>> {
    let metadata_file = metadata_dir.join("backup_metadata.json");
    
    if metadata_file.exists() {
        let file = File::open(metadata_file)?;
        let reader = BufReader::new(file);
        let metadata: BackupMetadata = serde_json::from_reader(reader)?;
        Ok(metadata)
    } else {
        Ok(BackupMetadata::default())
    }
}

pub fn save_backup_metadata(metadata_dir: &Path, metadata: &BackupMetadata) -> Result<(), Box<dyn std::error::Error>> {
    let metadata_file = metadata_dir.join("backup_metadata.json");
    let file = File::create(metadata_file)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, metadata)?;
    Ok(())
} 