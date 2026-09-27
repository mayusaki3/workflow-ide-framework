use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

pub fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;

    let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("wfide");
    let temp = parent.join(format!(".{file_name}.tmp"));

    {
        let mut file = File::create(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }

    #[cfg(windows)]
    if path.exists() {
        fs::remove_file(path)?;
    }

    fs::rename(&temp, path)?;

    if let Ok(directory) = File::open(parent) {
        let _ = directory.sync_all();
    }
    Ok(())
}
