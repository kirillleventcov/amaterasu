# 🔥 Amaterasu

> *"Amaterasu is the highest level of Fire Release. In most situations ... Amaterasu is impossible to avoid."*

A modern, fast file secure deletion tool for Linux. Named after the black flames that cannot be extinguished, Amaterasu ensures your data is irrecoverably destroyed.

## Features

- **🎯 Precise Deletion**: Files are overwritten with cryptographically secure random data
- **⚡ Storage-Aware**: Intelligent handling of HDDs, SSDs, and NVMe drives
- **🔒 Multiple Security Levels**: Fast, Standard, and Paranoid wiping modes
- **📊 Progress Tracking**: Visual progress bars for large files
- **✅ Verification**: Optional read-back verification to ensure complete data destruction
- **🚀 Modern Performance**: Async I/O and optimized block sizes

## Installation

```bash
git clone https://github.com/your-username/amaterasu
cd amaterasu
cargo build --release
```

## Usage

```bash
# Simple usage - wipe a single file
amaterasu secret.txt

# Wipe multiple files
amaterasu file1.txt file2.dat sensitive.pdf

# Use paranoid mode with verification
amaterasu --mode paranoid --verify important.doc

# Fast mode for quick deletion
amaterasu --mode fast temp_file.txt
```

### Wiping Modes

- **Fast** (1 pass): Single random overwrite - quick but basic
- **Standard** (3 passes): Random → Zeros → Random - balanced security
- **Paranoid** (7 passes): Multiple patterns including 0x55, 0xAA, 0xFF - maximum security

## Why Amaterasu?

Unlike traditional tools like `shred`, Amaterasu is built for modern storage systems:

- **SSD Optimized**: Single-pass wiping for SSDs (multiple passes don't improve security)
- **HDD Aware**: Multi-pass overwriting for traditional magnetic storage
- **Fast & Safe**: Rust's memory safety with high performance async I/O
- **Modern UX**: Clear progress indication and intelligent defaults

## Security Notes

⚠️ **Important**: No secure deletion tool can guarantee complete data destruction on all systems. Modern SSDs with wear leveling, copy-on-write filesystems (Btrfs, ZFS), and SSD over-provisioning can leave data remnants. For maximum security, use full-disk encryption.

## License

Licensed under GNU license.

