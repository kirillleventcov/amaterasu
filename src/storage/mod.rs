pub mod detector;

#[derive(Debug, Clone)]
pub enum StorageType {
    HDD { rotational: bool, block_size: usize },
    SSD { trim_support: bool },
    NVMe { optimal_io_size: usize },
    Unknown,
}

impl StorageType {
    pub fn get_optimal_block_size(&self) -> usize {
        match self {
            StorageType::HDD { block_size, .. } => *block_size,
            StorageType::SSD { .. } => 4096,
            StorageType::NVMe {
                optimal_io_size, ..
            } => *optimal_io_size,
            StorageType::Unknown => 4096,
        }
    }

    pub fn get_wipe_passes(&self) -> usize {
        match self {
            StorageType::HDD { .. } => 3,
            StorageType::SSD { .. } => 1,
            StorageType::NVMe { .. } => 1,
            StorageType::Unknown => 3,
        }
    }

    pub fn supports_secure_erase(&self) -> bool {
        matches!(self, StorageType::SSD { .. } | StorageType::NVMe { .. })
    }
}
