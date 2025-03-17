use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "linux_backup")]
#[command(about = "A backup tool for Linux directories", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, ValueEnum, Debug)]
pub enum BackupType {
    //full backup (copies all files)
    Full,
    //incremental backup (copies only files changed since last backup)
    Incremental,
    //differential backup (copies files that have changed since original backup)
    Differential,
}

#[derive(Subcommand)]
pub enum Commands {
    //backup specific directories
    Selective {
        //output file name
        #[arg(short, long, default_value = "backup.tar.gz")]
        output: String,

        //type of backup to perform
        #[arg(short, long, value_enum, default_value = "full")]
        backup_type: BackupType,
    },
    //backup entire system (excluding system directories)
    Full {
        //output file name
        #[arg(short, long, default_value = "system_backup.tar.gz")]
        output: String,

        //type of backup to perform
        #[arg(short, long, value_enum, default_value = "full")]
        backup_type: BackupType,
    },
    //restore from backup
    Restore {
        //backup file to restore from
        #[arg(short, long)]
        file: String,
        
        //directory to restore to
        #[arg(short, long, default_value = ".")]
        target: String,
    },
} 