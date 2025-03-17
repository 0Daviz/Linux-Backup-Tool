# 🐧 LBT (Linux Backup Tool) 🛠️
Perfect tool if you like to distrohop
LBT is a powerful and user-friendly backup tool designed specifically for Linux systems. Whether you need to back up selected directories or your entire system, LBT has got you covered! With support for **Full**, **Incremental**, and **Differential** backups, LBT ensures your data is safe and secure. 🚀

---

## 📦 Features

- **Backup Selected Directories**: Choose specific directories to back up with ease.
- **Backup Entire System**: Perform a full system backup (excluding system directories like `/proc`, `/sys`, etc.).
- **Restore Backup**: Restore your backups to any directory with a single command.
- **Backup Types**:
  - **Full Backup**: Backs up all selected files and directories.
  - **Incremental Backup**: Only backs up files changed since the last backup.
  - **Differential Backup**: Backs up files changed since the original backup.
- **Compression Levels**: Choose between **Fast**, **Default**, and **Best** compression levels for your backups.
- **Progress Bar**: Visual feedback with a progress bar during backup and restore operations.
- **Metadata Tracking**: Keeps track of backup history and timestamps for incremental and differential backups.

---

## 🛠️ Installation

To use LBT, you need to have **Rust** installed on your system. If you don't have Rust installed, you can install it by following the instructions on [rustup.rs](https://rustup.rs/).

Clone the repository:
```bash
git clone https://github.com/0Daviz/Linux-Backup-Tool.git
cd Linux-Backup-Tool
```
Build the project:
```bash
cargo build --release
```
Run the tool:
```bash
./target/release/lbt
   ```

📝 Backup Metadata

LBT stores backup metadata in the .linux_backup_metadata directory in your home folder. This metadata includes:

    Last Backup Time: Timestamp of the last backup.

    Original Backup Time: Timestamp of the original backup (for differential backups).

    Backup History: A record of all backups performed.

🛑 Exclusions

When performing a full system backup, LBT automatically excludes the following directories:

    /proc

    /sys

    /tmp

    /run

    /mnt

    /media

    /lost+found

    /dev

    /var/log

    /var/cache

    /var/tmp

    /root

    /home/*/.cache

📜 License

This project is licensed under the MIT License. See the LICENSE file for more details.
🙏 Contributing

Contributions are welcome! If you have any suggestions, bug reports, or feature requests, please open an issue or submit a pull request.
