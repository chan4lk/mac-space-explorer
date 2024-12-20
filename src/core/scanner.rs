use std::{path::{Path, PathBuf}, time::SystemTime};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub created: SystemTime,
    pub modified: SystemTime,
    pub is_dir: bool,
}

#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub total_files: usize,
    pub scanned_files: usize,
    pub current_path: Option<PathBuf>,
    pub total_size: u64,
}

impl Default for ScanProgress {
    fn default() -> Self {
        Self {
            total_files: 0,
            scanned_files: 0,
            current_path: None,
            total_size: 0,
        }
    }
}

pub fn get_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .map(|metadata| metadata.len())
        .sum()
}

pub fn scan_directory(path: &Path, progress: &mut ScanProgress) -> Vec<FileEntry> {
    let mut entries = Vec::new();
    
    // First count total files for progress
    progress.total_files = WalkDir::new(path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .count();
    
    progress.scanned_files = 0;
    
    for entry in WalkDir::new(path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        progress.current_path = Some(entry.path().to_path_buf());
        progress.scanned_files += 1;
        
        let metadata = entry.metadata().unwrap();
        let size = if metadata.is_file() {
            metadata.len()
        } else {
            get_dir_size(&entry.path())
        };
        
        progress.total_size += size;
        
        entries.push(FileEntry {
            path: entry.path().to_path_buf(),
            size,
            created: metadata.created().unwrap_or(SystemTime::now()),
            modified: metadata.modified().unwrap_or(SystemTime::now()),
            is_dir: metadata.is_dir(),
        });
    }
    
    entries
}
