use super::FilesystemOptimizer;
use crate::Result;
use std::path::Path;
use std::process::Command;

pub struct BtrfsOptimizer;

impl FilesystemOptimizer for BtrfsOptimizer {
    fn pre_wipe_setup(&self, path: &Path) -> Result<()> {
        // Try to disable CoW for better security
        println!(
            "⚠️  Btrfs detected - attempting to disable CoW for file: {}",
            path.display()
        );

        // Use chattr +C to disable CoW
        let output = Command::new("chattr").arg("+C").arg(path).output();

        match output {
            Ok(result) if result.status.success() => {
                println!("✅ CoW disabled for file");
            }
            Ok(result) => {
                println!(
                    "⚠️  Failed to disable CoW: {}",
                    String::from_utf8_lossy(&result.stderr)
                );
                println!("   Note: Wipe may be less effective on CoW filesystems");
            }
            Err(e) => {
                println!("⚠️  Could not run chattr command: {}", e);
                println!("   Note: Install e2fsprogs for better Btrfs support");
            }
        }

        Ok(())
    }

    fn post_wipe_cleanup(&self, _path: &Path) -> Result<()> {
        // Force defragmentation to ensure data is actually overwritten
        println!("🔄 Attempting filesystem sync for CoW cleanup...");

        // Use sync to ensure all data is written
        let _ = Command::new("sync").status();

        Ok(())
    }

    fn get_recommended_passes(&self) -> usize {
        // CoW filesystems may need additional passes for security
        5
    }

    fn should_disable_cow(&self) -> bool {
        true
    }
}

pub struct ZfsOptimizer;

impl FilesystemOptimizer for ZfsOptimizer {
    fn pre_wipe_setup(&self, path: &Path) -> Result<()> {
        println!(
            "⚠️  ZFS detected - CoW filesystem limitations apply: {}",
            path.display()
        );
        println!("   Note: ZFS snapshots may preserve deleted data");
        println!("   Recommendation: Remove relevant snapshots after wiping");
        Ok(())
    }

    fn post_wipe_cleanup(&self, _path: &Path) -> Result<()> {
        println!("🔄 Forcing ZFS sync...");
        let _ = Command::new("sync").status();
        Ok(())
    }

    fn get_recommended_passes(&self) -> usize {
        // Multiple passes may not be effective on ZFS due to CoW
        2
    }

    fn should_disable_cow(&self) -> bool {
        false // Cannot disable CoW on ZFS
    }
}
