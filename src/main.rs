mod backup;
mod restore;
mod utils;

use backup::{backup_selected_directories, backup_system};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use restore::restore_backup;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "\n===== LINUX BACKUP TOOL =====\n".green().bold());
    println!("A utility for backing up your Linux system");
    println!("{}", "--------------------------------\n".green());

    //main menu loop
    loop {
        let options = vec!["Backup Selected Directories", "Backup System", "Restore Backup", "Exit"];
        
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .default(0)
            .items(&options)
            .interact()?;
            
        match selection {
            0 => backup_selected_directories()?,
            1 => backup_system()?,
            2 => {
                //get backup file path
                let default_path = std::env::current_dir()?;
                let backup_file: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter path to backup file")
                    .default(default_path.to_string_lossy().to_string())
                    .interact_text()?;
                
                //get restore destination
                let restore_path: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter restore destination")
                    .default(".".to_string())
                    .interact_text()?;
                    
                restore_backup(&backup_file, &restore_path)?;
            },
            3 => {
                println!("Exiting...");
                break;
            },
            _ => unreachable!(),
        }
        
        println!("\nPress Enter to continue...");
        let _: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(" ")
            .allow_empty(true)
            .interact_text()?;
            
        //clear the screen
        print!("\x1B[2J\x1B[1;1H");
    }

    Ok(())
}
