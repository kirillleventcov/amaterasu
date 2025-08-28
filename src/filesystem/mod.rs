pub mod cow;
pub mod detector;

use crate::Result;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum FilesystemType {
    Ext4 { has_journal: bool },
    Btrfs { subvolume: bool },
    Xfs { realtime: bool },
    Zfs { compression: bool },
    F2fs,
    Unknown,
}

pub trait FilesystemOptimizer {
    fn pre_wipe_setup(&self, path: &Path) -> Result<()>;
    fn post_wipe_cleanup(&self, path: &Path) -> Result<()>;
    fn get_recommended_passes(&self) -> usize;
    fn should_disable_cow(&self) -> bool;
}

pub struct DefaultOptimizer;

impl FilesystemOptimizer for DefaultOptimizer {
    fn pre_wipe_setup(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn post_wipe_cleanup(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn get_recommended_passes(&self) -> usize {
        3
    }

    fn should_disable_cow(&self) -> bool {
        false
    }
}

impl FilesystemType {
    pub fn get_optimizer(&self) -> Box<dyn FilesystemOptimizer> {
        match self {
            FilesystemType::Btrfs { .. } => Box::new(cow::BtrfsOptimizer),
            FilesystemType::Zfs { .. } => Box::new(cow::ZfsOptimizer),
            FilesystemType::Ext4 { .. } => Box::new(detector::Ext4Optimizer),
            FilesystemType::Xfs { .. } => Box::new(detector::XfsOptimizer),
            FilesystemType::F2fs => Box::new(detector::F2fsOptimizer),
            FilesystemType::Unknown => Box::new(DefaultOptimizer),
        }
    }

    pub fn supports_cow(&self) -> bool {
        matches!(
            self,
            FilesystemType::Btrfs { .. } | FilesystemType::Zfs { .. }
        )
    }

    pub fn is_journaled(&self) -> bool {
        match self {
            FilesystemType::Ext4 { has_journal } => *has_journal,
            FilesystemType::Xfs { .. } => true,
            FilesystemType::F2fs => true,
            _ => false,
        }
    }
}
