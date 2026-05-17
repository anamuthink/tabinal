use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// A node in the file tree.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub depth: usize,
}

impl FileEntry {
    pub fn from_dir(path: &Path, depth: usize) -> Option<Self> {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        Some(Self {
            name,
            path: path.to_path_buf(),
            is_dir: path.is_dir(),
            depth,
        })
    }
}

/// Scan a directory and return sorted entries (dirs first, then files, alphabetical).
/// Maximum entries per directory to prevent DoS from huge directories.
const MAX_ENTRIES_PER_DIR: usize = 500;

fn scan_directory_filtered(path: &Path, depth: usize, show_hidden: bool) -> Vec<FileEntry> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut dirs = Vec::new();
    let mut files = Vec::new();
    let mut count = 0;

    for entry in entries.flatten() {
        if count >= MAX_ENTRIES_PER_DIR {
            break;
        }

        let entry_path = entry.path();
        let name = entry
            .file_name()
            .to_string_lossy()
            .to_string();

        // Always skip .git (too large and noisy)
        if name == ".git" {
            continue;
        }

        // Skip other hidden files/directories unless show_hidden is enabled
        if !show_hidden && name.starts_with('.') {
            continue;
        }

        // Skip symlinks to prevent traversal outside the project
        if let Ok(meta) = entry_path.symlink_metadata() {
            if meta.is_symlink() {
                continue;
            }
        }

        if let Some(file_entry) = FileEntry::from_dir(&entry_path, depth) {
            if file_entry.is_dir {
                dirs.push(file_entry);
            } else {
                files.push(file_entry);
            }
            count += 1;
        }
    }

    // Sort alphabetically (case-insensitive); cached key avoids repeated allocations
    dirs.sort_by_cached_key(|e| e.name.to_lowercase());
    files.sort_by_cached_key(|e| e.name.to_lowercase());

    // Directories first, then files
    dirs.extend(files);
    dirs
}

/// File tree state for the sidebar.
/// Interval between automatic rescans of visible directories.
const AUTO_REFRESH_INTERVAL_SECS: u64 = 2;

pub struct FileTree {
    pub root_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub show_hidden: bool,
    /// Flattened list of visible entries for rendering.
    flat_entries: Vec<FileEntry>,
    /// Timestamp of the last automatic rescan.
    last_refresh: Instant,
}

impl FileTree {
    /// Create a new file tree from a directory.
    pub fn new(root_path: PathBuf) -> Self {
        // Default: show hidden files (except .git)
        let root_path = root_path.canonicalize().unwrap_or(root_path);
        let entries = scan_directory_filtered(&root_path, 0, true);
        let mut tree = Self {
            root_path,
            entries,
            selected_index: 0,
            scroll_offset: 0,
            show_hidden: true,
            flat_entries: Vec::new(),
            last_refresh: Instant::now(),
        };
        tree.rebuild_flat();
        tree
    }

    /// Toggle showing hidden files and rescan.
    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.entries = scan_directory_filtered(&self.root_path, 0, self.show_hidden);
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.rebuild_flat();
    }

    /// Change the root to `new_root` and rescan. Resets selection and scroll.
    pub fn navigate_into(&mut self, new_root: PathBuf) {
        if !new_root.is_dir() {
            return;
        }
        let new_root = match new_root.canonicalize() {
            Ok(p) => p,
            Err(_) => new_root,
        };
        self.root_path = new_root;
        self.entries = scan_directory_filtered(&self.root_path, 0, self.show_hidden);
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.last_refresh = Instant::now();
        self.rebuild_flat();
    }

    /// Rebuild the flattened entry list from the tree structure.
    fn rebuild_flat(&mut self) {
        self.flat_entries.clear();
        for entry in &self.entries {
            self.flat_entries.push(entry.clone());
        }
    }

    /// Get the flattened list of visible entries.
    pub fn visible_entries(&self) -> &[FileEntry] {
        &self.flat_entries
    }

    /// Move selection up.
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down.
    pub fn move_down(&mut self) {
        if self.selected_index + 1 < self.flat_entries.len() {
            self.selected_index += 1;
        }
    }

    /// Right arrow: navigate into selected directory.
    pub fn move_into_child(&mut self) {
        let Some(entry) = self.flat_entries.get(self.selected_index).cloned() else {
            return;
        };
        if entry.is_dir {
            self.navigate_into(entry.path);
        }
    }

    /// Left arrow: navigate to parent directory, restoring cursor to the directory we came from.
    pub fn move_to_parent(&mut self) {
        let came_from = self.root_path.clone();
        if let Some(parent) = came_from.parent().map(|p| p.to_path_buf()) {
            self.navigate_into(parent);
            if let Some(idx) = self.flat_entries.iter().position(|e| e.path == came_from) {
                self.selected_index = idx;
            }
        }
    }

    /// Handle Enter key / mouse click on selected entry.
    /// Returns Some(path) if a file was selected for preview.
    /// If a directory is selected, navigates into it and returns None.
    pub fn toggle_or_select(&mut self) -> Option<PathBuf> {
        let flat_entry = self.flat_entries.get(self.selected_index).cloned()?;
        if flat_entry.is_dir {
            self.navigate_into(flat_entry.path);
            None
        } else {
            Some(flat_entry.path)
        }
    }

    /// Check if it's time to auto-refresh and do so if needed.
    /// Returns true if the tree was updated.
    pub fn auto_refresh_if_needed(&mut self) -> bool {
        if self.last_refresh.elapsed().as_secs() < AUTO_REFRESH_INTERVAL_SECS {
            return false;
        }
        self.last_refresh = Instant::now();

        // Remember the selected path so we can restore selection after rescan.
        let selected_path = self.flat_entries
            .get(self.selected_index)
            .map(|e| e.path.clone());

        self.entries = scan_directory_filtered(&self.root_path, 0, self.show_hidden);
        self.rebuild_flat();

        // Restore selection
        if let Some(ref path) = selected_path {
            if let Some(idx) = self.flat_entries.iter().position(|e| &e.path == path) {
                self.selected_index = idx;
            }
        }
        // Clamp selection
        if self.selected_index >= self.flat_entries.len() {
            self.selected_index = self.flat_entries.len().saturating_sub(1);
        }

        true
    }

    /// Adjust scroll offset to keep selected item visible.
    pub fn ensure_visible(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
        if self.selected_index >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected_index - visible_height + 1;
        }
    }

    /// Scroll up by amount.
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// Scroll down by amount.
    pub fn scroll_down(&mut self, amount: usize) {
        let max_offset = self.flat_entries.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + amount).min(max_offset);
    }
}
