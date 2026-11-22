//! File locks for runtime saves and logs. Rust's std file locking does not yet
//! support Android; Bionic exposes flock with the same descriptor-owned lifetime.
use std::fs::{File, TryLockError};
use std::io;

pub fn try_exclusive(file: &File) -> Result<(), TryLockError> {
    #[cfg(target_os = "android")]
    return flock(file, libc::LOCK_EX | libc::LOCK_NB).map_err(|error| {
        if error.kind() == io::ErrorKind::WouldBlock {
            TryLockError::WouldBlock
        } else {
            TryLockError::Error(error)
        }
    });
    #[cfg(not(target_os = "android"))]
    file.try_lock()
}

pub fn unlock(file: &File) -> io::Result<()> {
    #[cfg(target_os = "android")]
    return flock(file, libc::LOCK_UN);
    #[cfg(not(target_os = "android"))]
    file.unlock()
}

#[cfg(target_os = "android")]
fn flock(file: &File, operation: libc::c_int) -> io::Result<()> {
    use std::os::fd::AsRawFd;
    loop {
        // SAFETY: the borrowed File keeps its valid descriptor alive during flock.
        if unsafe { libc::flock(file.as_raw_fd(), operation) } == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    #[test]
    fn separate_opens_contend_and_explicit_unlock_releases_cloned_descriptor() {
        let root = std::env::var_os("STRATUM_LOCK_TEST_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let path = root.join(format!("stratum-lock-{}.tmp", std::process::id()));
        let first = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        let second = OpenOptions::new().write(true).open(&path).unwrap();
        try_exclusive(&first).unwrap();
        assert!(matches!(
            try_exclusive(&second),
            Err(TryLockError::WouldBlock)
        ));
        let clone = first.try_clone().unwrap();
        unlock(&first).unwrap();
        try_exclusive(&second).unwrap();
        drop(second);
        // Closing the last descriptor of the second open releases its lock too.
        try_exclusive(&first).unwrap();
        unlock(&first).unwrap();
        drop(clone);
        drop(first);
        std::fs::remove_file(path).unwrap();
    }
}
