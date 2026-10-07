//! Esportazione in sviluppo: i lettori Windows possono tenere i bindings mappati in memoria.

use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    time::Duration,
};

pub fn export<E: Error + 'static>(
    path: &Path,
    generate: impl FnOnce(&Path) -> Result<(), E>,
) -> Result<(), Box<dyn Error>> {
    let temporary = Temporary::new()?;
    generate(&temporary.0)?;
    let fresh = fs::read(&temporary.0)?;
    match retry_file_access(|| fs::read(path)) {
        Ok(current) if current == fresh => return Ok(()),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    retry_file_access(|| fs::write(path, &fresh))?;
    Ok(())
}

/// Solo i blocchi Windows di lettura/mappatura sono temporanei; gli altri errori restano immediati.
fn retry_file_access<T>(mut operation: impl FnMut() -> io::Result<T>) -> io::Result<T> {
    for attempt in 0..=20 {
        match operation() {
            Err(error)
                if cfg!(windows)
                    && matches!(error.raw_os_error(), Some(32 | 33 | 1224))
                    && attempt < 20 =>
            {
                std::thread::sleep(Duration::from_millis(100));
            }
            result => return result,
        }
    }
    unreachable!()
}

struct Temporary(PathBuf);

impl Temporary {
    fn new() -> io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "memotape-bindings-export-{}-{}.ts",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(io::Error::other)?
                .as_nanos()
        ));
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        Ok(Self(path))
    }
}

impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::export;
    use std::{fs, io, path::PathBuf};

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "memotape-bindings-{}-{}.ts",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::write(&path, "originale").unwrap();
            Self(path)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    #[test]
    fn bindings_invariati_non_vengono_riscritti() {
        let file = Fixture::new();
        let before = fs::metadata(&file.0).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        export(&file.0, |path| fs::write(path, "originale")).unwrap();
        assert_eq!(fs::metadata(&file.0).unwrap().modified().unwrap(), before);
    }

    #[test]
    fn bindings_cambiati_vengono_aggiornati() {
        let file = Fixture::new();
        export(&file.0, |path| fs::write(path, "aggiornati")).unwrap();
        assert_eq!(fs::read_to_string(&file.0).unwrap(), "aggiornati");
    }

    #[test]
    fn errore_generazione_conserva_i_bindings() {
        let file = Fixture::new();
        let error = export(&file.0, |path| {
            fs::write(path, "incompleti")?;
            Err(io::Error::other("generazione fallita"))
        });
        assert!(error.is_err());
        assert_eq!(fs::read_to_string(&file.0).unwrap(), "originale");
    }

    #[cfg(windows)]
    #[test]
    fn bindings_invariati_con_mappatura_windows_1224() {
        use std::{ffi::c_void, os::windows::io::AsRawHandle};

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn CreateFileMappingW(
                file: *mut c_void,
                attributes: *const c_void,
                protection: u32,
                size_high: u32,
                size_low: u32,
                name: *const u16,
            ) -> *mut c_void;
            fn MapViewOfFile(
                mapping: *mut c_void,
                access: u32,
                offset_high: u32,
                offset_low: u32,
                length: usize,
            ) -> *mut c_void;
            fn UnmapViewOfFile(view: *const c_void) -> i32;
            fn CloseHandle(handle: *mut c_void) -> i32;
        }

        let file = Fixture::new();
        let reader = fs::File::open(&file.0).unwrap();
        // PAGE_READONLY e FILE_MAP_READ: manteniamo aperta una vera vista Windows.
        let mapping = unsafe {
            CreateFileMappingW(
                reader.as_raw_handle(),
                std::ptr::null(),
                2,
                0,
                0,
                std::ptr::null(),
            )
        };
        assert!(!mapping.is_null());
        let view = unsafe { MapViewOfFile(mapping, 4, 0, 0, 0) };
        assert!(!view.is_null());
        let direct = fs::write(&file.0, "originale").unwrap_err();
        assert_eq!(direct.raw_os_error(), Some(1224));
        let result = export(&file.0, |path| fs::write(path, "originale"));
        let changed = export(&file.0, |path| fs::write(path, "aggiornati"));
        unsafe {
            UnmapViewOfFile(view);
            CloseHandle(mapping);
        }
        result.unwrap();
        let error = changed.unwrap_err().downcast::<io::Error>().unwrap();
        assert_eq!(error.raw_os_error(), Some(1224));
        assert_eq!(fs::read_to_string(&file.0).unwrap(), "originale");
    }

    #[cfg(windows)]
    #[test]
    fn bindings_attendono_un_lettore_temporaneo() {
        use std::os::windows::fs::OpenOptionsExt;

        let file = Fixture::new();
        let reader = fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&file.0)
            .unwrap();
        let release = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(150));
            drop(reader);
        });
        let result = export(&file.0, |path| fs::write(path, "aggiornati"));
        release.join().unwrap();
        result.unwrap();
        assert_eq!(fs::read_to_string(&file.0).unwrap(), "aggiornati");
    }
}
