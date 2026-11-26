use crate::error::RecoveryError;
use crate::snapshot::Snapshot;
use std::fs;
use std::path::PathBuf;

/// Validates on-disk snapshot directories produced by `RecoveryManager`.
///
/// Actual restore is performed by `RecoveryManager::restore_snapshot`,
/// which handles pre-restore backups and selective data restore.
pub struct RestoreManager;

impl RestoreManager {
    pub fn new() -> Result<Self, RecoveryError> {
        Ok(Self)
    }

    /// Restore is intentionally not implemented here — it is destructive
    /// and must go through `RecoveryManager::restore_snapshot`, which
    /// takes a pre-restore backup unless `force` is set.
    pub fn restore(&self, _snapshot_path: &PathBuf) -> Result<(), RecoveryError> {
        Err(RecoveryError::RestoreFailed(
            "use RecoveryManager::restore_snapshot (takes a pre-restore backup)".into(),
        ))
    }

    /// Verify a snapshot directory: must contain a parseable
    /// `metadata.json` whose `id` matches the directory name, plus at
    /// least one payload file.
    pub fn verify_snapshot(&self, snapshot_path: &PathBuf) -> Result<bool, RecoveryError> {
        if !snapshot_path.is_dir() {
            return Ok(false);
        }
        let meta_path = snapshot_path.join("metadata.json");
        let Ok(text) = fs::read_to_string(&meta_path) else {
            return Ok(false);
        };
        let Ok(meta) = serde_json::from_str::<Snapshot>(&text) else {
            return Ok(false);
        };
        let dir_name = snapshot_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if meta.id != dir_name {
            return Ok(false);
        }
        // A snapshot with no payload files is not restorable.
        let has_payload = fs::read_dir(snapshot_path)?
            .filter_map(|e| e.ok())
            .any(|e| e.file_name() != "metadata.json");
        Ok(has_payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::Path;

    fn snapshot_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("restore-test-{}-{}", std::process::id(), tag));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_meta(dir: &Path, id: &str) {
        let meta = Snapshot {
            id: id.into(),
            name: "t".into(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".into(),
            size: 0,
            include_user_data: false,
            include_packages: false,
        };
        fs::write(
            dir.join("metadata.json"),
            serde_json::to_string(&meta).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn verify_accepts_valid_snapshot() {
        let root = snapshot_dir("valid");
        let snap = root.join("snap1");
        fs::create_dir_all(&snap).unwrap();
        write_meta(&snap, "snap1");
        fs::write(snap.join("system-config.tar"), b"x").unwrap();
        assert!(RestoreManager::new()
            .unwrap()
            .verify_snapshot(&snap)
            .unwrap());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn verify_rejects_bad_snapshots() {
        let m = RestoreManager::new().unwrap();
        let root = snapshot_dir("bad");
        // missing dir
        assert!(!m.verify_snapshot(&root.join("nope")).unwrap());
        // metadata only, no payload
        let empty = root.join("empty");
        fs::create_dir_all(&empty).unwrap();
        write_meta(&empty, "empty");
        assert!(!m.verify_snapshot(&empty).unwrap());
        // id mismatch
        let mism = root.join("dirA");
        fs::create_dir_all(&mism).unwrap();
        write_meta(&mism, "other");
        fs::write(mism.join("payload"), b"x").unwrap();
        assert!(!m.verify_snapshot(&mism).unwrap());
        // malformed metadata
        let corrupt = root.join("corrupt");
        fs::create_dir_all(&corrupt).unwrap();
        let mut f = fs::File::create(corrupt.join("metadata.json")).unwrap();
        f.write_all(b"{not json").unwrap();
        fs::write(corrupt.join("payload"), b"x").unwrap();
        assert!(!m.verify_snapshot(&corrupt).unwrap());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn restore_fails_loudly() {
        let err = RestoreManager::new()
            .unwrap()
            .restore(&PathBuf::from("/tmp/x"))
            .unwrap_err();
        assert!(matches!(err, RecoveryError::RestoreFailed(_)));
    }
}
