use std::path::{Path, PathBuf};

/// Structure publique `FileBrowser`
pub struct FileBrowser {
    /// Champ public `current_dir` de la structure correspondante.
    pub current_dir: PathBuf,
    /// Champ public `entries` de la structure correspondante.
    pub entries: Vec<FileEntry>,
    /// Champ public `selected` de la structure correspondante.
    pub selected: usize,
    /// Champ public `filter_extensions` de la structure correspondante.
    pub filter_extensions: Vec<String>,
}

/// Structure publique `FileEntry`
pub struct FileEntry {
    /// Champ public `path` de la structure correspondante.
    pub path: PathBuf,
    /// Champ public `is_dir` de la structure correspondante.
    pub is_dir: bool,
    /// Champ public `size` de la structure correspondante.
    pub size: u64,
}

impl FileBrowser {
    /// Fonction publique `new`
    pub fn new(start_dir: &Path, extensions: Vec<String>) -> Self {
        let mut browser = Self {
            current_dir: start_dir.to_path_buf(),
            entries: Vec::new(),
            selected: 0,
            filter_extensions: extensions,
        };
        browser.refresh();
        browser
    }

    /// Fonction publique `refresh`
    pub fn refresh(&mut self) {
        self.entries.clear();
        self.selected = 0;

        // Add parent directory entry
        if let Some(parent) = self.current_dir.parent() {
            self.entries.push(FileEntry {
                path: parent.to_path_buf(),
                is_dir: true,
                size: 0,
            });
        }

        let Ok(read_dir) = std::fs::read_dir(&self.current_dir) else {
            return;
        };

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in read_dir.flatten() {
            let path = entry.path();
            let is_dir = path.is_dir();
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);

            if is_dir {
                dirs.push(FileEntry { path, is_dir, size });
            } else if self.matches_filter(&path) {
                files.push(FileEntry { path, is_dir, size });
            }
        }

        dirs.sort_by(|a, b| a.path.cmp(&b.path));
        files.sort_by(|a, b| a.path.cmp(&b.path));

        self.entries.extend(dirs);
        self.entries.extend(files);
    }

    fn matches_filter(&self, path: &Path) -> bool {
        if self.filter_extensions.is_empty() {
            return true;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            return false;
        };
        let ext_lower = ext.to_lowercase();
        self.filter_extensions
            .iter()
            .any(|f| f.to_lowercase() == ext_lower)
    }

    /// Fonction publique `enter_selected`
    pub fn enter_selected(&mut self) {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                self.current_dir = entry.path.clone();
                self.refresh();
            }
        }
    }

    /// Fonction publique `selected_path`
    pub fn selected_path(&self) -> Option<&Path> {
        self.entries
            .get(self.selected)
            .filter(|e| !e.is_dir)
            .map(|e| e.path.as_path())
    }
}
