use crate::error::RecoveryError;
use crate::snapshot::Snapshot;
use crate::restore::RestoreManager;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;

pub struct RecoveryManager {
    snapshot_dir: PathBuf,
    restore_manager: RestoreManager,
}

impl RecoveryManager {
    pub fn new() -> Result<Self, RecoveryError> {
        let snapshot_dir = directories::ProjectDirs::from("org", "sigmaos", "sigmaos")
            .map(|p| p.data_dir().join("snapshots"))
            .unwrap_or_else(|| PathBuf::from("~/.local/share/sigmaos/snapshots"));
        
        fs::create_dir_all(&snapshot_dir)?;
        
        Ok(Self {
            snapshot_dir,
            restore_manager: RestoreManager::new()?,
        })
    }
    
    pub fn create_snapshot(&self, name: &str, description: Option<String>, include_user_data: bool, include_packages: bool) -> Result<Snapshot, RecoveryError> {
        let id = self.generate_snapshot_id()?;
        let snapshot_path = self.snapshot_dir.join(&id);
        
        fs::create_dir_all(&snapshot_path)?;
        
        let snapshot = Snapshot {
            id: id.clone(),
            name: name.to_string(),
            description,
            created_at: chrono::Utc::now().to_rfc3339(),
            size: 0,
            include_user_data,
            include_packages,
        };
        
        self.backup_system_config(&snapshot_path)?;
        
        if include_user_data {
            self.backup_user_data(&snapshot_path)?;
        }
        
        if include_packages {
            self.backup_package_list(&snapshot_path)?;
        }
        
        let size = self.calculate_directory_size(&snapshot_path)?;
        
        let metadata_path = snapshot_path.join("metadata.json");
        let metadata_content = serde_json::to_string_pretty(&snapshot)?;
        fs::write(&metadata_path, metadata_content)?;
        
        log::info!("Created snapshot {} with size {} bytes", name, size);
        
        Ok(snapshot)
    }
    
    pub fn list_snapshots(&self) -> Result<Vec<Snapshot>, RecoveryError> {
        let mut snapshots = Vec::new();
        
        for entry in fs::read_dir(&self.snapshot_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let metadata_path = path.join("metadata.json");
                if metadata_path.exists() {
                    let metadata_content = fs::read_to_string(&metadata_path)?;
                    let snapshot: Snapshot = serde_json::from_str(&metadata_content)?;
                    snapshots.push(snapshot);
                }
            }
        }
        
        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(snapshots)
    }
    
    pub fn get_snapshot_info(&self, snapshot_id: &str) -> Result<Snapshot, RecoveryError> {
        let snapshots = self.list_snapshots()?;
        
        snapshots.into_iter()
            .find(|s| s.id == snapshot_id || s.name == snapshot_id)
            .ok_or_else(|| RecoveryError::SnapshotNotFound(snapshot_id.to_string()))
    }
    
    pub fn restore_snapshot(&self, snapshot_id: &str, force: bool) -> Result<(), RecoveryError> {
        let snapshot = self.get_snapshot_info(snapshot_id)?;
        let snapshot_path = self.snapshot_dir.join(&snapshot.id);
        
        if !snapshot_path.exists() {
            return Err(RecoveryError::SnapshotNotFound(snapshot_id.to_string()));
        }
        
        if !force {
            self.create_snapshot("pre-restore", Some("Automatic backup before restore".to_string()), true, true)?;
        }
        
        self.restore_system_config(&snapshot_path)?;
        
        if snapshot.include_user_data {
            self.restore_user_data(&snapshot_path)?;
        }
        
        if snapshot.include_packages {
            self.restore_packages(&snapshot_path)?;
        }
        
        log::info!("Restored snapshot {}", snapshot.name);
        
        Ok(())
    }
    
    pub fn delete_snapshot(&self, snapshot_id: &str, force: bool) -> Result<(), RecoveryError> {
        let snapshot = self.get_snapshot_info(snapshot_id)?;
        let snapshot_path = self.snapshot_dir.join(&snapshot.id);
        
        if !snapshot_path.exists() {
            return Err(RecoveryError::SnapshotNotFound(snapshot_id.to_string()));
        }
        
        if force {
            fs::remove_dir_all(&snapshot_path)?;
            log::info!("Deleted snapshot {}", snapshot.name);
        } else {
            let trash_dir = directories::ProjectDirs::from("org", "sigmaos", "sigmaos")
                .map(|p| p.data_dir().join("trash"))
                .unwrap_or_else(|| PathBuf::from("~/.local/share/sigmaos/trash"));
            fs::create_dir_all(&trash_dir)?;
            fs::rename(&snapshot_path, trash_dir.join(&snapshot.id))?;
            log::info!("Moved snapshot {} to trash", snapshot.name);
        }
        
        Ok(())
    }
    
    pub fn get_status(&self) -> Result<RecoveryStatus, RecoveryError> {
        let snapshots = self.list_snapshots()?;
        let total_size = self.calculate_total_size()?;
        let last_snapshot = snapshots.first().map(|s| s.created_at.clone()).unwrap_or_default();
        
        Ok(RecoveryStatus {
            snapshot_count: snapshots.len(),
            total_size,
            last_snapshot,
            snapshots,
        })
    }
    
    fn generate_snapshot_id(&self) -> Result<String, RecoveryError> {
        let timestamp = chrono::Utc::now().timestamp();
        let random = rand::random::<u32>();
        Ok(format!("snapshot-{}-{:08x}", timestamp, random))
    }
    
    fn backup_system_config(&self, snapshot_path: &Path) -> Result<(), RecoveryError> {
        let config_dir = directories::ProjectDirs::from("org", "sigmaos", "sigmaos")
            .map(|p| p.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("~/.config/sigmaos"));
        
        let backup_dir = snapshot_path.join("config");
        fs::create_dir_all(&backup_dir)?;
        
        if config_dir.exists() {
            self.copy_directory(&config_dir, &backup_dir)?;
        }
        
        let system_config_dir = PathBuf::from("/etc/sigmaos");
        if system_config_dir.exists() {
            let system_backup = snapshot_path.join("system-config");
            fs::create_dir_all(&system_backup)?;
            self.copy_directory(&system_config_dir, &system_backup)?;
        }
        
        Ok(())
    }
    
    fn backup_user_data(&self, snapshot_path: &Path) -> Result<(), RecoveryError> {
        let user_data_dir = directories::ProjectDirs::from("org", "sigmaos", "sigmaos")
            .map(|p| p.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("~/.local/share/sigmaos"));
        
        let backup_dir = snapshot_path.join("user-data");
        fs::create_dir_all(&backup_dir)?;
        
        if user_data_dir.exists() {
            self.copy_directory(&user_data_dir, &backup_dir)?;
        }
        
        Ok(())
    }
    
    fn backup_package_list(&self, snapshot_path: &Path) -> Result<(), RecoveryError> {
        let package_list_path = snapshot_path.join("packages.txt");
        
        let output = std::process::Command::new("pacman")
            .arg("-Q")
            .arg("--explicit")
            .output()?;
        
        if output.status.success() {
            fs::write(&package_list_path, &output.stdout)?;
        }
        
        Ok(())
    }
    
    fn restore_system_config(&self, snapshot_path: &Path) -> Result<(), RecoveryError> {
        let config_backup = snapshot_path.join("config");
        let config_dir = directories::ProjectDirs::from("org", "sigmaos", "sigmaos")
            .map(|p| p.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("~/.config/sigmaos"));
        
        if config_backup.exists() {
            let current_backup = config_dir.join(".backup-before-restore");
            if config_dir.exists() {
                self.copy_directory(&config_dir, &current_backup)?;
            }
            
            fs::create_dir_all(&config_dir)?;
            self.copy_directory(&config_backup, &config_dir)?;
        }
        
        let system_config_backup = snapshot_path.join("system-config");
        let system_config_dir = PathBuf::from("/etc/sigmaos");
        
        if system_config_backup.exists() {
            fs::create_dir_all(&system_config_dir)?;
            self.copy_directory(&system_config_backup, &system_config_dir)?;
        }
        
        Ok(())
    }
    
    fn restore_user_data(&self, snapshot_path: &Path) -> Result<(), RecoveryError> {
        let user_data_backup = snapshot_path.join("user-data");
        let user_data_dir = directories::ProjectDirs::from("org", "sigmaos", "sigmaos")
            .map(|p| p.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("~/.local/share/sigmaos"));
        
        if user_data_backup.exists() {
            let current_backup = user_data_dir.join(".backup-before-restore");
            if user_data_dir.exists() {
                self.copy_directory(&user_data_dir, &current_backup)?;
            }
            
            fs::create_dir_all(&user_data_dir)?;
            self.copy_directory(&user_data_backup, &user_data_dir)?;
        }
        
        Ok(())
    }
    
    fn restore_packages(&self, snapshot_path: &Path) -> Result<(), RecoveryError> {
        let package_list_path = snapshot_path.join("packages.txt");
        
        if package_list_path.exists() {
            let package_list = fs::read_to_string(&package_list_path)?;
            let packages: Vec<&str> = package_list.lines().collect();
            
            for package in packages {
                if !package.is_empty() {
                    let status = std::process::Command::new("sudo")
                        .arg("pacman")
                        .arg("-S")
                        .arg("--noconfirm")
                        .arg(package)
                        .status()?;
                    
                    if !status.success() {
                        log::warn!("Failed to install package: {}", package);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn copy_directory(&self, source: &Path, dest: &Path) -> Result<(), RecoveryError> {
        if source.exists() {
            for entry in WalkDir::new(source) {
                let entry = entry?;
                let path = entry.path();
                let relative_path = path.strip_prefix(source)?;
                let dest_path = dest.join(relative_path);
                
                if entry.file_type().is_dir() {
                    fs::create_dir_all(&dest_path)?;
                } else if entry.file_type().is_file() {
                    if let Some(parent) = dest_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(path, &dest_path)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn calculate_directory_size(&self, dir: &Path) -> Result<u64, RecoveryError> {
        let mut size = 0;
        
        for entry in WalkDir::new(dir) {
            let entry = entry?;
            if entry.file_type().is_file() {
                size += entry.metadata()?.len();
            }
        }
        
        Ok(size)
    }
    
    fn calculate_total_size(&self) -> Result<u64, RecoveryError> {
        let mut total_size = 0;
        
        for entry in fs::read_dir(&self.snapshot_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                total_size += self.calculate_directory_size(&path)?;
            }
        }
        
        Ok(total_size)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStatus {
    pub snapshot_count: usize,
    pub total_size: u64,
    pub last_snapshot: String,
    pub snapshots: Vec<Snapshot>,
}
