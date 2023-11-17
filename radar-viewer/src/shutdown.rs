use std::{net::{TcpStream, Shutdown}, sync::{Arc, atomic::{AtomicBool, Ordering}}, fs::File};

use fd_lock::{RwLock, RwLockWriteGuard};

use crate::util;

pub struct LockFile<'a> {
    lock_file: RwLock<File>,
    lock_file_guard: RwLockWriteGuard<'a, File>,
}
impl<'a> LockFile<'a> {
    pub fn initialise() -> Self {
        // Attempt to obtain exclusive write access to the lockfile, creating it if it is not there.
        // If this fails, another instance of this application is already running so we close.
        let lock_file_path = util::get_config_dir().unwrap().join(".radarlockfile");
        let mut lock_file = fd_lock::RwLock::new(File::create(lock_file_path).expect("Unable to create lock file"));
        let lock_file_guard = lock_file.try_write().expect("Another instance of this application is already running. Closing...");


        Self { lock_file, lock_file_guard }

    }
}