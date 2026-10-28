use crate::{codec, lua::Value, Result};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub struct Document {
    pub path: PathBuf,
    pub data: Value,
    original: Value,
    original_bytes: Vec<u8>,
    pub last_backup: Option<PathBuf>,
}

pub fn read_limited(path: &Path) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| format!("Cannot open {}: {e}", path.display()))?
        .take(codec::MAX_COMPRESSED as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > codec::MAX_COMPRESSED {
        return Err("File exceeds the 16 MiB limit.".into());
    }
    Ok(bytes)
}

impl Document {
    pub fn open(path: &Path) -> Result<Self> {
        let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
        if !path.is_file() {
            return Err("Choose a regular save file.".into());
        }
        let bytes = read_limited(&path)?;
        let data = codec::decode(&bytes)?;
        Ok(Self {
            path,
            original: data.clone(),
            data,
            original_bytes: bytes,
            last_backup: None,
        })
    }
    pub fn dirty(&self) -> bool {
        self.data != self.original
    }
    pub fn revert(&mut self) {
        self.data = self.original.clone();
    }

    /// Only the opened source may be overwritten. Save As always creates a new file.
    pub fn save(&mut self) -> Result<PathBuf> {
        let bytes = codec::encode(&self.data)?;
        if read_limited(&self.path)? != self.original_bytes {
            return Err("The save changed on disk. Close Balatro, reopen the file, and reapply your edits; or use Save copy with a new filename.".into());
        }
        let parent = self.path.parent().ok_or("File has no parent directory")?;
        let permissions = fs::metadata(&self.path)
            .map_err(|e| e.to_string())?
            .permissions();
        if permissions.readonly() {
            return Err("The source is read-only; use Save copy.".into());
        }
        let prefix = format!(
            "{}.backup-",
            self.path.file_name().unwrap().to_string_lossy()
        );
        let mut backup = tempfile::Builder::new()
            .prefix(&prefix)
            .tempfile_in(parent)
            .map_err(|e| e.to_string())?;
        backup
            .write_all(&self.original_bytes)
            .map_err(|e| e.to_string())?;
        backup.as_file().sync_all().map_err(|e| e.to_string())?;
        let (_, backup_path) = backup.keep().map_err(|e| e.to_string())?;
        self.last_backup = Some(backup_path.clone());
        let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        temporary.write_all(&bytes).map_err(|e| e.to_string())?;
        temporary
            .as_file()
            .set_permissions(permissions)
            .map_err(|e| e.to_string())?;
        temporary.as_file().sync_all().map_err(|e| e.to_string())?;
        if read_limited(&self.path)? != self.original_bytes {
            return Err(
                "The save changed during writing; the replacement was canceled. Reopen the save."
                    .into(),
            );
        }
        temporary.persist(&self.path).map_err(|e| {
            format!(
                "Could not replace save: {e}. Backup: {}",
                backup_path.display()
            )
        })?;
        sync_directory(parent);
        self.original_bytes = bytes;
        self.original = self.data.clone();
        Ok(backup_path)
    }

    pub fn save_copy(&mut self, target: &Path) -> Result<()> {
        let bytes = codec::encode(&self.data)?;
        create_file(target, &bytes)?;
        self.path = fs::canonicalize(target).map_err(|e| e.to_string())?;
        self.original_bytes = bytes;
        self.original = self.data.clone();
        self.last_backup = None;
        Ok(())
    }
}

pub fn create_file(target: &Path, bytes: &[u8]) -> Result<()> {
    let parent = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temporary.write_all(bytes).map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary.persist_noclobber(target).map_err(|e| {
        format!(
            "Could not create {}: {e}. Choose a filename that does not already exist.",
            target.display()
        )
    })?;
    sync_directory(parent);
    Ok(())
}

fn sync_directory(path: &Path) {
    #[cfg(unix)]
    if let Ok(directory) = File::open(path) {
        let _ = directory.sync_all();
    }
    #[cfg(not(unix))]
    let _ = path;
}
