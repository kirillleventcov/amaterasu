use super::{FilesystemOptimizer, FilesystemType};
use crate::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn detect_filesystem_type(path: &Path) -> Result<FilesystemType> {
    // Try to get filesystem type using /proc/mounts
    let mounts = fs::read_to_string("/proc/mounts")?;
    let device = get_device_for_path(path)?;

    for line in mounts.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 3 && fields[0].contains(&device) {
            let fs_type = fields[2];
            return parse_filesystem_type(fs_type, &fields[0]);
        }
    }

    // Fallback to stat command
    match Command::new("stat")
        .arg("-f")
        .arg("-c")
        .arg("%T")
        .arg(path)
        .output()
    {
        Ok(output) if output.status.success() => {
            let fs_type = String::from_utf8_lossy(&output.stdout);
            parse_filesystem_type(fs_type.trim(), "unknown")
        }
        _ => Ok(FilesystemType::Unknown),
    }
}

fn get_device_for_path(path: &Path) -> Result<String> {
    match Command::new("df").arg("-T").arg(path).output() {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = output_str.lines().collect();
            if lines.len() > 1 {
                let fields: Vec<&str> = lines[1].split_whitespace().collect();
                if !fields.is_empty() {
                    return Ok(fields[0].to_string());
                }
            }
            Ok("unknown".to_string())
        }
        _ => Ok("unknown".to_string()),
    }
}

fn parse_filesystem_type(fs_type: &str, device: &str) -> Result<FilesystemType> {
    match fs_type.to_lowercase().as_str() {
        "ext4" => {
            let has_journal = check_ext4_journal(device);
            Ok(FilesystemType::Ext4 { has_journal })
        }
        "btrfs" => {
            let subvolume = device.contains("subvol");
            Ok(FilesystemType::Btrfs { subvolume })
        }
        "xfs" => {
            let realtime = check_xfs_realtime(device);
            Ok(FilesystemType::Xfs { realtime })
        }
        "zfs" => {
            let compression = check_zfs_compression(device);
            Ok(FilesystemType::Zfs { compression })
        }
        "f2fs" => Ok(FilesystemType::F2fs),
        _ => Ok(FilesystemType::Unknown),
    }
}

fn check_ext4_journal(device: &str) -> bool {
    // Check if ext4 has journaling enabled
    match Command::new("dumpe2fs").arg("-h").arg(device).output() {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            output_str.contains("has_journal")
        }
        _ => true, // Assume journal is enabled by default
    }
}

fn check_xfs_realtime(_device: &str) -> bool {
    // For now, assume no realtime volumes
    false
}

fn check_zfs_compression(_device: &str) -> bool {
    // For now, assume compression is enabled
    true
}

// Filesystem-specific optimizers
pub struct Ext4Optimizer;

impl FilesystemOptimizer for Ext4Optimizer {
    fn pre_wipe_setup(&self, path: &Path) -> Result<()> {
        println!("📁 Ext4 filesystem detected: {}", path.display());
        println!("   Journaling considerations apply");
        Ok(())
    }

    fn post_wipe_cleanup(&self, _path: &Path) -> Result<()> {
        // Sync to ensure journal writes are flushed
        let _ = Command::new("sync").status();
        Ok(())
    }

    fn get_recommended_passes(&self) -> usize {
        3
    }

    fn should_disable_cow(&self) -> bool {
        false
    }
}

pub struct XfsOptimizer;

impl FilesystemOptimizer for XfsOptimizer {
    fn pre_wipe_setup(&self, path: &Path) -> Result<()> {
        println!("📁 XFS filesystem detected: {}", path.display());
        Ok(())
    }

    fn post_wipe_cleanup(&self, _path: &Path) -> Result<()> {
        let _ = Command::new("sync").status();
        Ok(())
    }

    fn get_recommended_passes(&self) -> usize {
        3
    }

    fn should_disable_cow(&self) -> bool {
        false
    }
}

pub struct F2fsOptimizer;

impl FilesystemOptimizer for F2fsOptimizer {
    fn pre_wipe_setup(&self, path: &Path) -> Result<()> {
        println!("📁 F2FS filesystem detected: {}", path.display());
        println!("   Flash-friendly filesystem optimizations apply");
        Ok(())
    }

    fn post_wipe_cleanup(&self, _path: &Path) -> Result<()> {
        let _ = Command::new("sync").status();
        Ok(())
    }

    fn get_recommended_passes(&self) -> usize {
        1 // F2FS is designed for flash storage
    }

    fn should_disable_cow(&self) -> bool {
        false
    }
}
