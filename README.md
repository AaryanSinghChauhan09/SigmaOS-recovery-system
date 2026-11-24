# SigmaOS Recovery System

SigmaOS system recovery and rollback system with snapshot management.

## Features

- **System Snapshots** - Create snapshots of system state
- **User Data Backup** - Backup user data and configurations
- **Package Tracking** - Track installed packages
- **Restore** - Restore system from snapshots
- **Rollback** - Rollback to previous states
- **Verification** - Snapshot integrity verification
- **Compression** - Compressed snapshot storage
- **Incremental** - Incremental snapshot support

## Installation

```bash
cargo install sigmaos-recovery-system
```

## Usage

### Create Snapshot

```bash
sigmaos-recovery snapshot my-backup --description "Before major update"
sigmaos-recovery snapshot my-backup --include-user-data --include-packages
```

### List Snapshots

```bash
sigmaos-recovery list
sigmaos-recovery list --detailed
```

### Restore Snapshot

```bash
sigmaos-recovery restore my-backup
sigmaos-recovery restore my-backup --dry-run
sigmaos-recovery restore my-backup --force
```

### Delete Snapshot

```bash
sigmaos-recovery delete my-backup
sigmaos-recovery delete my-backup --force
```

### Show Status

```bash
sigmaos-recovery status
sigmaos-recovery status --detailed
```

## Snapshot Types

### System Snapshot

Captures system configuration and state:

- System configuration files
- Package lists
- Boot configuration
- Service configurations

### User Snapshot

Captures user data and preferences:

- User configuration files
- User data and documents
- Application settings
- Desktop preferences

### Full Snapshot

Complete system backup including:

- System configuration
- User data
- Package lists
- Application data
- System state

## Snapshot Process

### Creation Phase

1. Generate unique snapshot ID
2. Create snapshot directory
3. Backup system configuration
4. Backup user data (if requested)
5. Backup package list (if requested)
6. Calculate snapshot size
7. Save metadata

### Storage Phase

1. Compress snapshot data
2. Store in snapshot directory
3. Update snapshot index
4. Clean up temporary files

### Restore Phase

1. Verify snapshot integrity
2. Create pre-restore backup
3. Restore system configuration
4. Restore user data (if included)
5. Restore packages (if included)
6. Verify restoration

## Configuration

### Snapshot Configuration

```toml
[snapshots]
max_snapshots = 10
max_size = 10737418240  # 10GB
compression = "zstd"
encryption = true
retention_days = 30
```

### Storage Configuration

```toml
[storage]
snapshot_dir = "/var/lib/sigmaos/snapshots"
backup_dir = "/var/lib/sigmaos/backups"
temp_dir = "/tmp/sigmaos-snapshots"
```

## Security

### Encryption

- Snapshots encrypted at rest
- Secure key management
- Password-protected snapshots
- Secure deletion

### Access Control

- Root-only snapshot creation
- User-specific snapshots
- Permission-based access
- Audit logging

## Verification

### Integrity Checks

- SHA-256 hash verification
- File integrity checks
- Metadata validation
- Corruption detection

### Recovery Verification

- Boot verification
- Service verification
- Configuration validation
- User data validation

## Error Handling

### Snapshot Errors

- Insufficient disk space
- Permission errors
- Corruption errors
- Verification errors

### Recovery

- Automatic backup before restore
- Rollback on failure
- Partial restore recovery
- Manual intervention options

## License

GPL-3.0-or-later
