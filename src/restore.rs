use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn restore_backup(backup_file: &str, target_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "\n---- Restore Backup ----".blue().bold());
    println!("Restoring from backup: {} to {}", backup_file, target_dir);
    
    //validate backup file exists
    if !Path::new(backup_file).exists() {
        println!("{}", format!("Error: Backup file does not exist: {}", backup_file).red());
        return Ok(());
    }
    
    //create target directory if it doesn't exist
    fs::create_dir_all(target_dir)?;
    
    //create progress bar
    let progress = ProgressBar::new_spinner();
    progress.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap());
    progress.set_message("Extracting files...");
    
    //use tar command for extraction as it handles permissions better than rust libraries
    let status = Command::new("tar")
        .arg("-xzf")
        .arg(backup_file)
        .arg("-C")
        .arg(target_dir)
        .status()?;
    
    progress.finish();
    
    if status.success() {
        println!("{}", "Restore completed successfully!".green().bold());
        println!("Files restored to: {}", target_dir);
    } else {
        println!("{}", format!("Restore failed with exit code: {}", status).red());
    }
    
    Ok(())
} 