use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

pub fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("wfide");
    let temp = parent.join(format!(".{file_name}.tmp"));

    {
        let mut file = File::create(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }

    replace_file(&temp, path)?;

    if let Ok(directory) = File::open(parent) {
        let _ = directory.sync_all();
    }
    Ok(())
}

#[cfg(not(windows))]
fn replace_file(temp: &Path, path: &Path) -> io::Result<()> {
    fs::rename(temp, path)
}

#[cfg(windows)]
fn replace_file(temp: &Path, path: &Path) -> io::Result<()> {
    if !path.exists() {
        return fs::rename(temp, path);
    }

    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::ReplaceFileW;

    let replaced: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let replacement: Vec<u16> = temp.as_os_str().encode_wide().chain(Some(0)).collect();

    let result = unsafe {
        ReplaceFileW(
            replaced.as_ptr(),
            replacement.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
