use clap::{Parser, Subcommand};
use sigmaos_recovery_system::{RecoveryError, RecoveryManager};

#[derive(Parser)]
#[command(name = "sigmaos-recovery")]
#[command(about = "SigmaOS system recovery and rollback system")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create system snapshot
    Snapshot {
        /// Snapshot name
        name: String,
        /// Snapshot description
        #[arg(short, long)]
        description: Option<String>,
        /// Include user data
        #[arg(short, long)]
        include_user_data: bool,
        /// Include system packages
        #[arg(short, long)]
        include_packages: bool,
    },
    /// List available snapshots
    List {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Restore system from snapshot
    Restore {
        /// Snapshot name or ID
        snapshot: String,
        /// Dry run
        #[arg(short, long)]
        dry_run: bool,
        /// Force restore
        #[arg(short, long)]
        force: bool,
    },
    /// Delete snapshot
    Delete {
        /// Snapshot name or ID
        snapshot: String,
        /// Force delete
        #[arg(short, long)]
        force: bool,
    },
    /// Show recovery status
    Status {
        /// Show detailed status
        #[arg(short, long)]
        detailed: bool,
    },
}

fn main() -> Result<(), RecoveryError> {
    env_logger::init();

    let cli = Cli::parse();
    let recovery_manager = RecoveryManager::new()?;

    match cli.command {
        Commands::Snapshot {
            name,
            description,
            include_user_data,
            include_packages,
        } => {
            let snapshot = recovery_manager.create_snapshot(
                &name,
                description,
                include_user_data,
                include_packages,
            )?;
            println!("Snapshot created successfully");
            println!("Name: {}", snapshot.name);
            println!("ID: {}", snapshot.id);
            println!("Size: {} bytes", snapshot.size);
            println!("Created: {}", snapshot.created_at);
        }
        Commands::List { detailed } => {
            let snapshots = recovery_manager.list_snapshots()?;
            if snapshots.is_empty() {
                println!("No snapshots available");
            } else {
                println!("{} snapshot(s) available:", snapshots.len());
                for snapshot in snapshots {
                    if detailed {
                        println!("  {} ({})", snapshot.name, snapshot.id);
                        println!("    Size: {} bytes", snapshot.size);
                        println!("    Created: {}", snapshot.created_at);
                        if let Some(desc) = snapshot.description {
                            println!("    Description: {}", desc);
                        }
                    } else {
                        println!("  {}", snapshot.name);
                    }
                }
            }
        }
        Commands::Restore {
            snapshot,
            dry_run,
            force,
        } => {
            if dry_run {
                let snapshot_info = recovery_manager.get_snapshot_info(&snapshot)?;
                println!("Dry run - would restore snapshot:");
                println!("  Name: {}", snapshot_info.name);
                println!("  ID: {}", snapshot_info.id);
                println!("  Size: {} bytes", snapshot_info.size);
                println!("  Created: {}", snapshot_info.created_at);
            } else {
                recovery_manager.restore_snapshot(&snapshot, force)?;
                println!("System restored from snapshot {}", snapshot);
            }
        }
        Commands::Delete { snapshot, force } => {
            recovery_manager.delete_snapshot(&snapshot, force)?;
            println!("Snapshot {} deleted", snapshot);
        }
        Commands::Status { detailed } => {
            let status = recovery_manager.get_status()?;
            println!("Recovery system status:");
            println!("  Snapshots: {}", status.snapshot_count);
            println!("  Total size: {} bytes", status.total_size);
            println!("  Last snapshot: {}", status.last_snapshot);

            if detailed {
                println!("\nSnapshot details:");
                for snapshot in status.snapshots {
                    println!("  {} - {}", snapshot.name, snapshot.created_at);
                }
            }
        }
    }

    Ok(())
}
