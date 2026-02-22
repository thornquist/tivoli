use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::Connection;

use crate::errors::AppError;

pub struct InMemoryDb {
    conn: Arc<Mutex<Connection>>,
    disk_path: PathBuf,
}

impl InMemoryDb {
    pub fn load_from_disk(path: &str) -> Self {
        let disk_path = PathBuf::from(path);
        let disk_conn = Connection::open(&disk_path).expect("Failed to open disk database");
        let mut mem_conn = Connection::open_in_memory().expect("Failed to open in-memory database");

        // Copy disk DB into memory
        {
            let backup = rusqlite::backup::Backup::new(&disk_conn, &mut mem_conn)
                .expect("Failed to init backup");
            backup
                .run_to_completion(5000, std::time::Duration::ZERO, None)
                .expect("Failed to backup disk DB into memory");
        }

        mem_conn
            .execute_batch("PRAGMA cache_size = -64000;")
            .expect("Failed to set pragmas");

        tracing::info!(
            "Loaded database into memory from {}",
            disk_path.display()
        );

        InMemoryDb {
            conn: Arc::new(Mutex::new(mem_conn)),
            disk_path,
        }
    }

    pub fn conn(&self) -> Result<MutexGuard<'_, Connection>, AppError> {
        self.conn
            .lock()
            .map_err(|e| AppError::DbError(format!("Mutex poisoned: {e}")))
    }

    pub fn flush_to_disk(&self) -> Result<(), String> {
        let mem_conn = self
            .conn
            .lock()
            .map_err(|e| format!("Mutex poisoned: {e}"))?;
        let mut disk_conn =
            Connection::open(&self.disk_path).map_err(|e| format!("Failed to open disk DB: {e}"))?;

        let backup = rusqlite::backup::Backup::new(&*mem_conn, &mut disk_conn)
            .map_err(|e| format!("Failed to init backup: {e}"))?;
        backup
            .run_to_completion(5000, std::time::Duration::ZERO, None)
            .map_err(|e| format!("Failed to flush to disk: {e}"))?;

        tracing::info!("Flushed database to disk");
        Ok(())
    }
}

impl Clone for InMemoryDb {
    fn clone(&self) -> Self {
        InMemoryDb {
            conn: Arc::clone(&self.conn),
            disk_path: self.disk_path.clone(),
        }
    }
}
