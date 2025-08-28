use super::StorageType;
use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

pub fn detect_storage_type(path: &Path) -> Result<StorageType> {
    let device = get_device_for_path(path)?;
    detect_storage_for_device(&device)
}

fn get_device_for_path(path: &Path) -> Result<String> {
    let metadata = fs::metadata(path).context("Failed to get file metadata")?;
    let dev = metadata.dev();

    let major = (dev >> 8) & 0xff;
    let minor = dev & 0xff;

    let device_name = if major == 8 {
        format!("sd{}", char::from(b'a' + (minor / 16) as u8))
    } else if major == 259 {
        format!("nvme{}n{}", minor / 2, (minor % 2) + 1)
    } else {
        return Ok("unknown".to_string());
    };

    Ok(device_name)
}

fn detect_storage_for_device(device: &str) -> Result<StorageType> {
    if device == "unknown" {
        return Ok(StorageType::Unknown);
    }

    if device.starts_with("nvme") {
        let optimal_io_size =
            read_sys_value(&format!("/sys/block/{}/queue/optimal_io_size", device)).unwrap_or(4096);
        return Ok(StorageType::NVMe { optimal_io_size });
    }

    let rotational_path = format!("/sys/block/{}/queue/rotational", device);
    let rotational = read_sys_value(&rotational_path).unwrap_or(1) == 1;

    if rotational {
        let block_size = read_sys_value(&format!("/sys/block/{}/queue/logical_block_size", device))
            .unwrap_or(512);
        Ok(StorageType::HDD {
            rotational: true,
            block_size,
        })
    } else {
        let trim_support = check_trim_support(device);
        Ok(StorageType::SSD { trim_support })
    }
}

fn read_sys_value(path: &str) -> Option<usize> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn check_trim_support(device: &str) -> bool {
    let discard_path = format!("/sys/block/{}/queue/discard_granularity", device);
    read_sys_value(&discard_path).unwrap_or(0) > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_detection() {
        if let Ok(storage_type) = detect_storage_for_device("sda") {
            println!("Storage type: {:?}", storage_type);
        }
    }
}
