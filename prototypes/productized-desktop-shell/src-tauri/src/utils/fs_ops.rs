use std::fs;
use std::path::Path;

#[cfg(test)]
use std::path::PathBuf;

pub(crate) fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("remove file failed {}: {error}", path.display())),
    }
}

#[cfg(test)]
pub(crate) fn fixture_dir(stage: &str, name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(stage)
        .join(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn remove_file_if_exists_missing_file_is_ok() {
        let missing = std::env::temp_dir().join(format!("u3-missing-{}", unique_nanos()));

        assert!(remove_file_if_exists(&missing).is_ok());
    }

    #[test]
    fn remove_file_if_exists_deletes_existing_file() {
        let path = std::env::temp_dir().join(format!("u3-remove-{}", unique_nanos()));
        fs::write(&path, b"delete me").expect("write temp file");

        remove_file_if_exists(&path).expect("remove temp file");

        assert!(!path.exists());
    }

    #[test]
    fn remove_file_if_exists_keeps_other_io_errors() {
        let path = std::env::temp_dir().join(format!("u3-remove-dir-{}", unique_nanos()));
        fs::create_dir_all(&path).expect("create temp dir");

        let err = remove_file_if_exists(&path).expect_err("directory removal should fail");

        assert!(err.starts_with(&format!("remove file failed {}:", path.display())));
        fs::remove_dir_all(&path).expect("cleanup temp dir");
    }

    #[test]
    fn fixture_dir_joins_stage_and_name() {
        let path = fixture_dir("r3-a1", "valid-workflow-core");

        assert!(path.ends_with(
            Path::new("fixtures")
                .join("r3-a1")
                .join("valid-workflow-core")
        ));
    }

    fn unique_nanos() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    }
}
