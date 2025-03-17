use crate::utils::{self, BackupMetadata, BackupType};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use flate2::write::GzEncoder;
use flate2::Compression;
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};
use tar::Builder;
use walkdir::WalkDir;

pub fn backup_selected_directories() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "\n---- Backup Selected Directories ----".blue().bold());

    //list of common home directories
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let mut options = vec![
        format!("{}", home_dir.join("Documents").display()),
        format!("{}", home_dir.join("Pictures").display()),
        format!("{}", home_dir.join("Videos").display()),
        format!("{}", home_dir.join("Music").display()),
        format!("{}", home_dir.join("Downloads").display()),
        format!("{}", home_dir.join(".config").display()),
        format!("{}", home_dir.join(".local/share").display()),
        "/etc".to_string(),
    ];
    
    //add "Custom directory" option
    options.push("Enter a custom directory path".to_string());
    
    println!("{}", "IMPORTANT: Use SPACEBAR to select directories, then press ENTER to confirm".yellow().bold());
    
    //ask user to select directories with clear instructions
    let selection = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select directories to backup (SPACEBAR to select, ENTER to confirm)")
        .items(&options)
        .interact()?;
    
    let mut selected_dirs: Vec<String> = Vec::new();
    
    //process selected directories
    for i in selection {
        if i == options.len() - 1 {
            //custom directory was selected
            let custom_path: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter custom directory path")
                .interact_text()?;
                
            if !custom_path.is_empty() {
                if Path::new(&custom_path).exists() {
                    selected_dirs.push(custom_path);
                } else {
                    println!("{}", format!("Warning: Path does not exist: {}", custom_path).yellow());
                }
            }
        } else {
            selected_dirs.push(options[i].clone());
        }
    }

    if selected_dirs.is_empty() {
        println!("{}", "No directories selected, returning to main menu.".yellow());
        return Ok(());
    }

    println!("Selected directories: {:?}", selected_dirs);

    //ask for backup type
    let backup_types = vec!["Full", "Incremental", "Differential"];
    let selected_type = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select backup type")
        .default(0)
        .items(&backup_types)
        .interact()?;
        
    let backup_type = match selected_type {
        0 => BackupType::Full,
        1 => BackupType::Incremental,
        2 => BackupType::Differential,
        _ => BackupType::Full,
    };

    //ask for compression level
    let compression_levels = vec!["Fast (1)", "Default (6)", "Best (9)"];
    let selected_level = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select compression level")
        .default(1)
        .items(&compression_levels)
        .interact()?;

    let compression = match selected_level {
        0 => Compression::fast(),
        1 => Compression::default(),
        2 => Compression::best(),
        _ => Compression::default(),
    };

    //ask for output file location
    let default_name = format!("backup_{}.tar.gz", chrono::Local::now().format("%Y%m%d_%H%M%S"));
    let output: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter output file name")
        .default(default_name)
        .interact_text()?;

    //create absolute path for output
    let output_path = if Path::new(&output).is_absolute() {
        PathBuf::from(output)
    } else {
        std::env::current_dir()?.join(output)
    };
    
    //ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    //create output file
    let file = File::create(&output_path)?;
    let encoder = GzEncoder::new(file, compression);
    let mut archive = Builder::new(encoder);

    //load or create backup metadata
    let metadata_path = home_dir.join(utils::METADATA_DIR);
    fs::create_dir_all(&metadata_path)?;
    
    let mut metadata = utils::load_backup_metadata(&metadata_path)?;

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    
    if metadata.original_backup_time.is_none() {
        metadata.original_backup_time = Some(current_time);
    }

    let start_time = Instant::now();
    
    //process each selected directory based on backup type
    for dir in selected_dirs {
        match backup_type {
            BackupType::Full => {
                backup_directory(&mut archive, &dir)?;
            },
            BackupType::Incremental => {
                incremental_backup(&mut archive, &dir, &metadata, current_time)?;
            },
            BackupType::Differential => {
                differential_backup(&mut archive, &dir, &metadata, current_time)?;
            }
        }
    }

    //finish the archive
    archive.finish()?;
    
    //update metadata
    metadata.last_backup_time = Some(current_time);
    utils::save_backup_metadata(&metadata_path, &metadata)?;
    
    let duration = start_time.elapsed();
    println!("\n{}", "Backup completed!".green().bold());
    println!("Time taken: {:.2} seconds", duration.as_secs_f64());
    println!("Backup saved to: {}", output_path.display().to_string().green());

    Ok(())
}

pub fn backup_system() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "\n---- Backup System ----".blue().bold());
    
    //check if running as root
    let is_root = unsafe { libc::geteuid() == 0 };
    
    if !is_root {
        println!("{}", "Warning: Not running as root. Some system files may not be accessible.".yellow());
        println!("For a complete system backup, consider running the program with sudo.");
        
        let options = vec!["Continue without root", "Return to main menu"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .default(0)
            .items(&options)
            .interact()?;
            
        if selection == 1 {
            return Ok(());
        }
    }

    //ask for backup type
    let backup_types = vec!["Full", "Incremental", "Differential"];
    let selected_type = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select backup type")
        .default(0)
        .items(&backup_types)
        .interact()?;
        
    let backup_type = match selected_type {
        0 => BackupType::Full,
        1 => BackupType::Incremental,
        2 => BackupType::Differential,
        _ => BackupType::Full,
    };

    //ask for compression level
    let compression_levels = vec!["Fast (1)", "Default (6)", "Best (9)"];
    let selected_level = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select compression level")
        .default(1)
        .items(&compression_levels)
        .interact()?;

    let compression = match selected_level {
        0 => Compression::fast(),
        1 => Compression::default(),
        2 => Compression::best(),
        _ => Compression::default(),
    };

    //ask for output file location
    let default_name = format!("system_backup_{}.tar.gz", chrono::Local::now().format("%Y%m%d_%H%M%S"));
    let output: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter output file name")
        .default(default_name)
        .interact_text()?;

    //create absolute path for output
    let output_path = if Path::new(&output).is_absolute() {
        PathBuf::from(output)
    } else {
        std::env::current_dir()?.join(output)
    };
    
    //ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    //create output file
    let file = File::create(&output_path)?;
    let encoder = GzEncoder::new(file, compression);
    let mut archive = Builder::new(encoder);

    let start_time = Instant::now();
    
    //directories to exclude from full backup
    let exclude_dirs = vec![
        "/proc", "/sys", "/tmp", "/run", "/mnt", "/media", 
        "/lost+found", "/dev", "/var/log", "/var/cache",
        "/var/tmp", "/root", "/home/*/.cache",
    ];
    
    //home directory gets different backup types
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let metadata_path = home_dir.join(utils::METADATA_DIR);
    fs::create_dir_all(&metadata_path)?;
    
    let mut metadata = utils::load_backup_metadata(&metadata_path)?;
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    
    if metadata.original_backup_time.is_none() {
        metadata.original_backup_time = Some(current_time);
    }
    
    match backup_type {
        BackupType::Full => {
            //backup home directory with exclusions
            backup_with_exclusions(&mut archive, "/home", &exclude_dirs)?;
            
            //backup important system directories
            backup_with_exclusions(&mut archive, "/etc", &exclude_dirs)?;
            backup_with_exclusions(&mut archive, "/usr/local", &exclude_dirs)?;
            
            if is_root {
                //these directories typically need root access
                backup_with_exclusions(&mut archive, "/var", &exclude_dirs)?;
                backup_with_exclusions(&mut archive, "/opt", &exclude_dirs)?;
            }
        },
        BackupType::Incremental => {
            //incremental backup for home with exclusions
            incremental_backup_with_exclusions(&mut archive, "/home", &exclude_dirs, &metadata, current_time)?;
            
            //important system directories
            incremental_backup_with_exclusions(&mut archive, "/etc", &exclude_dirs, &metadata, current_time)?;
            incremental_backup_with_exclusions(&mut archive, "/usr/local", &exclude_dirs, &metadata, current_time)?;
            
            if is_root {
                incremental_backup_with_exclusions(&mut archive, "/var", &exclude_dirs, &metadata, current_time)?;
                incremental_backup_with_exclusions(&mut archive, "/opt", &exclude_dirs, &metadata, current_time)?;
            }
        },
        BackupType::Differential => {
            //differential backup for home with exclusions
            differential_backup_with_exclusions(&mut archive, "/home", &exclude_dirs, &metadata, current_time)?;
            
            //important system directories
            differential_backup_with_exclusions(&mut archive, "/etc", &exclude_dirs, &metadata, current_time)?;
            differential_backup_with_exclusions(&mut archive, "/usr/local", &exclude_dirs, &metadata, current_time)?;
            
            if is_root {
                differential_backup_with_exclusions(&mut archive, "/var", &exclude_dirs, &metadata, current_time)?;
                differential_backup_with_exclusions(&mut archive, "/opt", &exclude_dirs, &metadata, current_time)?;
            }
        }
    }
    
    //finish the archive
    archive.finish()?;
    
    //update metadata
    metadata.last_backup_time = Some(current_time);
    utils::save_backup_metadata(&metadata_path, &metadata)?;
    
    let duration = start_time.elapsed();
    println!("\n{}", "Backup completed!".green().bold());
    println!("Time taken: {:.2} seconds", duration.as_secs_f64());
    println!("Backup saved to: {}", output_path.display().to_string().green());

    Ok(())
}

fn backup_directory(archive: &mut Builder<GzEncoder<File>>, dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Backing up directory: {}", dir_path);
    
    let path = Path::new(dir_path);
    if !path.exists() {
        println!("{}", format!("Warning: Path does not exist: {}", dir_path).yellow());
        return Ok(());
    }
    
    let total_files = WalkDir::new(dir_path).into_iter().count();
    let progress = ProgressBar::new(total_files as u64);
    progress.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap()
        .progress_chars("#>-"));
    
    for entry in WalkDir::new(dir_path) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                let name = path.strip_prefix("/").unwrap_or(path);
                
                if path.is_file() {
                    match File::open(path) {
                        Ok(mut file) => {
                            archive.append_file(name, &mut file)?;
                        }
                        Err(e) => {
                            println!("{}", format!("Warning: Could not open file {}: {}", path.display(), e).yellow());
                        }
                    }
                } else if path.is_dir() && entry.depth() > 0 {
                    archive.append_dir(name, path)?;
                }
                
                progress.inc(1);
            }
            Err(e) => {
                println!("{}", format!("Warning: Error accessing entry: {}", e).yellow());
            }
        }
    }
    
    progress.finish();
    Ok(())
}

fn incremental_backup(
    archive: &mut Builder<GzEncoder<File>>, 
    dir_path: &str,
    metadata: &BackupMetadata,
    current_time: u64
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Performing incremental backup of: {}", dir_path);
    
    let path = Path::new(dir_path);
    if !path.exists() {
        println!("{}", format!("Warning: Path does not exist: {}", dir_path).yellow());
        return Ok(());
    }
    
    let last_backup_time = metadata.last_backup_time.unwrap_or(0);
    let progress = ProgressBar::new_spinner();
    progress.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap());
    
    progress.set_message(format!("Checking for changes in {} since last backup", dir_path));
    
    let mut files_backed_up = 0;
    
    for entry in WalkDir::new(dir_path) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                
                if path.is_file() {
                    //check if file was modified since last backup
                    if let Ok(metadata) = path.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(modified_secs) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                                if modified_secs.as_secs() > last_backup_time {
                                    let name = path.strip_prefix("/").unwrap_or(path);
                                    progress.set_message(format!("Adding {}", path.display()));
                                    
                                    match File::open(path) {
                                        Ok(mut file) => {
                                            archive.append_file(name, &mut file)?;
                                            files_backed_up += 1;
                                        }
                                        Err(e) => {
                                            println!("{}", format!("Warning: Could not open file {}: {}", path.display(), e).yellow());
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if path.is_dir() && entry.depth() > 0 {
                    //only add directories that are new
                    if let Ok(dir_metadata) = path.metadata() {
                        if let Ok(created) = dir_metadata.created() {
                            if let Ok(created_secs) = created.duration_since(SystemTime::UNIX_EPOCH) {
                                if created_secs.as_secs() > last_backup_time {
                                    let name = path.strip_prefix("/").unwrap_or(path);
                                    archive.append_dir(name, path)?;
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("{}", format!("Warning: Error accessing entry: {}", e).yellow());
            }
        }
    }
    
    progress.finish_with_message(format!("Incremental backup of {} completed. {} files backed up.", dir_path, files_backed_up));
    
    Ok(())
}

fn differential_backup(
    archive: &mut Builder<GzEncoder<File>>, 
    dir_path: &str,
    metadata: &BackupMetadata,
    current_time: u64
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Performing differential backup of: {}", dir_path);
    
    let path = Path::new(dir_path);
    if !path.exists() {
        println!("{}", format!("Warning: Path does not exist: {}", dir_path).yellow());
        return Ok(());
    }
    
    let original_backup_time = metadata.original_backup_time.unwrap_or(0);
    let progress = ProgressBar::new_spinner();
    progress.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap());
    
    progress.set_message(format!("Checking for changes in {} since original backup", dir_path));
    
    let mut files_backed_up = 0;
    
    for entry in WalkDir::new(dir_path) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                
                if path.is_file() {
                    //check if file was modified since original backup
                    if let Ok(metadata) = path.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(modified_secs) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                                if modified_secs.as_secs() > original_backup_time {
                                    let name = path.strip_prefix("/").unwrap_or(path);
                                    progress.set_message(format!("Adding {}", path.display()));
                                    
                                    match File::open(path) {
                                        Ok(mut file) => {
                                            archive.append_file(name, &mut file)?;
                                            files_backed_up += 1;
                                        }
                                        Err(e) => {
                                            println!("{}", format!("Warning: Could not open file {}: {}", path.display(), e).yellow());
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if path.is_dir() && entry.depth() > 0 {
                    //only add directories that are new since original backup
                    if let Ok(dir_metadata) = path.metadata() {
                        if let Ok(created) = dir_metadata.created() {
                            if let Ok(created_secs) = created.duration_since(SystemTime::UNIX_EPOCH) {
                                if created_secs.as_secs() > original_backup_time {
                                    let name = path.strip_prefix("/").unwrap_or(path);
                                    archive.append_dir(name, path)?;
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("{}", format!("Warning: Error accessing entry: {}", e).yellow());
            }
        }
    }
    
    progress.finish_with_message(format!("Differential backup of {} completed. {} files backed up.", dir_path, files_backed_up));
    
    Ok(())
}

fn backup_with_exclusions(archive: &mut Builder<GzEncoder<File>>, dir_path: &str, exclusions: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Backing up directory with exclusions: {}", dir_path);
    
    let path = Path::new(dir_path);
    if !path.exists() {
        println!("{}", format!("Warning: Path does not exist: {}", dir_path).yellow());
        return Ok(());
    }
    
    let progress = ProgressBar::new_spinner();
    progress.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap());
    progress.set_message(format!("Processing {}", dir_path));
    
    for entry in WalkDir::new(dir_path).into_iter().filter_entry(|e| {
        let path = e.path().to_string_lossy();
        !exclusions.iter().any(|ex| {
            if ex.contains('*') {
                //handle glob patterns
                if let Ok(pattern) = glob(ex) {
                    pattern.into_iter().any(|p| {
                        if let Ok(p) = p {
                            path.starts_with(p.to_string_lossy().as_ref())
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            } else {
                //simple prefix matching
                path.starts_with(ex)
            }
        })
    }) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                let name = path.strip_prefix("/").unwrap_or(path);
                
                progress.set_message(format!("Adding {}", path.display()));
                
                if path.is_file() {
                    match File::open(path) {
                        Ok(mut file) => {
                            archive.append_file(name, &mut file)?;
                        }
                        Err(e) => {
                            println!("{}", format!("Warning: Could not open file {}: {}", path.display(), e).yellow());
                        }
                    }
                } else if path.is_dir() && entry.depth() > 0 {
                    archive.append_dir(name, path)?;
                }
            }
            Err(e) => {
                println!("{}", format!("Warning: Error accessing entry: {}", e).yellow());
            }
        }
    }
    
    progress.finish_with_message(format!("Completed {}", dir_path));
    Ok(())
}

fn incremental_backup_with_exclusions(
    archive: &mut Builder<GzEncoder<File>>, 
    dir_path: &str, 
    exclusions: &[&str],
    metadata: &BackupMetadata,
    current_time: u64
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Performing incremental backup with exclusions: {}", dir_path);
    
    let path = Path::new(dir_path);
    if !path.exists() {
        println!("{}", format!("Warning: Path does not exist: {}", dir_path).yellow());
        return Ok(());
    }
    
    let last_backup_time = metadata.last_backup_time.unwrap_or(0);
    let progress = ProgressBar::new_spinner();
    progress.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap());
    
    progress.set_message(format!("Checking for changes in {} since last backup", dir_path));
    
    let mut files_backed_up = 0;
    
    for entry in WalkDir::new(dir_path).into_iter().filter_entry(|e| {
        let path = e.path().to_string_lossy();
        !exclusions.iter().any(|ex| {
            if ex.contains('*') {
                //handle glob patterns
                if let Ok(pattern) = glob(ex) {
                    pattern.into_iter().any(|p| {
                        if let Ok(p) = p {
                            path.starts_with(p.to_string_lossy().as_ref())
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            } else {
                //simple prefix matching
                path.starts_with(ex)
            }
        })
    }) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                
                if path.is_file() {
                    //check if file was modified since last backup
                    if let Ok(metadata) = path.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(modified_secs) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                                if modified_secs.as_secs() > last_backup_time {
                                    let name = path.strip_prefix("/").unwrap_or(path);
                                    progress.set_message(format!("Adding {}", path.display()));
                                    
                                    match File::open(path) {
                                        Ok(mut file) => {
                                            archive.append_file(name, &mut file)?;
                                            files_backed_up += 1;
                                        }
                                        Err(e) => {
                                            println!("{}", format!("Warning: Could not open file {}: {}", path.display(), e).yellow());
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if path.is_dir() && entry.depth() > 0 {
                    //only add directories that are new
                    if let Ok(dir_metadata) = path.metadata() {
                        if let Ok(created) = dir_metadata.created() {
                            if let Ok(created_secs) = created.duration_since(SystemTime::UNIX_EPOCH) {
                                if created_secs.as_secs() > last_backup_time {
                                    let name = path.strip_prefix("/").unwrap_or(path);
                                    archive.append_dir(name, path)?;
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("{}", format!("Warning: Error accessing entry: {}", e).yellow());
            }
        }
    }
    
    progress.finish_with_message(format!("Incremental backup of {} completed. {} files backed up.", dir_path, files_backed_up));
    
    Ok(())
}

fn differential_backup_with_exclusions(
    archive: &mut Builder<GzEncoder<File>>, 
    dir_path: &str, 
    exclusions: &[&str],
    metadata: &BackupMetadata,
    current_time: u64
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Performing differential backup with exclusions: {}", dir_path);
    
    let path = Path::new(dir_path);
    if !path.exists() {
        println!("{}", format!("Warning: Path does not exist: {}", dir_path).yellow());
        return Ok(());
    }
    
    let original_backup_time = metadata.original_backup_time.unwrap_or(0);
    let progress = ProgressBar::new_spinner();
    progress.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap());
    
    progress.set_message(format!("Checking for changes in {} since original backup", dir_path));
    
    let mut files_backed_up = 0;
    
    for entry in WalkDir::new(dir_path).into_iter().filter_entry(|e| {
        let path = e.path().to_string_lossy();
        !exclusions.iter().any(|ex| {
            if ex.contains('*') {
                //handle glob patterns
                if let Ok(pattern) = glob(ex) {
                    pattern.into_iter().any(|p| {
                        if let Ok(p) = p {
                            path.starts_with(p.to_string_lossy().as_ref())
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            } else {
                //simple prefix matching
                path.starts_with(ex)
            }
        })
    }) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                
                if path.is_file() {
                    //check if file was modified since original backup
                    if let Ok(metadata) = path.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(modified_secs) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                                if modified_secs.as_secs() > original_backup_time {
                                    let name = path.strip_prefix("/").unwrap_or(path);
                                    progress.set_message(format!("Adding {}", path.display()));
                                    
                                    match File::open(path) {
                                        Ok(mut file) => {
                                            archive.append_file(name, &mut file)?;
                                            files_backed_up += 1;
                                        }
                                        Err(e) => {
                                            println!("{}", format!("Warning: Could not open file {}: {}", path.display(), e).yellow());
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if path.is_dir() && entry.depth() > 0 {
                    //only add directories that are new since original backup
                    if let Ok(dir_metadata) = path.metadata() {
                        if let Ok(created) = dir_metadata.created() {
                            if let Ok(created_secs) = created.duration_since(SystemTime::UNIX_EPOCH) {
                                if created_secs.as_secs() > original_backup_time {
                                    let name = path.strip_prefix("/").unwrap_or(path);
                                    archive.append_dir(name, path)?;
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("{}", format!("Warning: Error accessing entry: {}", e).yellow());
            }
        }
    }
    
    progress.finish_with_message(format!("Differential backup of {} completed. {} files backed up.", dir_path, files_backed_up));
    
    Ok(())
} 