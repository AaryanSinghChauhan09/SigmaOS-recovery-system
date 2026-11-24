use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecoveryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("TOML error: {0}")]
    Toml(#[from] toml::ser::Error),
    
    #[error("WalkDir error: {0}")]
    WalkDir(#[from] walkdir::Error),
    
    #[error("StripPrefix error: {0}")]
    StripPrefix(#[from] std::path::StripPrefixError),
    
    #[error("Recovery error: {0}")]
    Recovery(String),
    
    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),
    
    #[error("Invalid snapshot: {0}")]
    InvalidSnapshot(String),
    
    #[error("Restore failed: {0}")]
    RestoreFailed(String),
    
    #[error("Backup failed: {0}")]
    BackupFailed(String),
}
