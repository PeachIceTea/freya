use std::path::{Path, PathBuf};

use crate::api::response::ApiError;

/// Canonicalizes a path and verifies it stays within allowed bounds.
/// Both the path and the root are canonicalized, so symlinks in either are resolved.
/// Fails if either path doesn't exist or if the path escapes the allowed directory.
pub fn validate_path_within_bounds(path: &Path, allowed_root: &Path) -> Result<PathBuf, ApiError> {
    let canonical_root = std::fs::canonicalize(allowed_root).map_err(|_| ApiError::InvalidPath)?;
    let canonical = std::fs::canonicalize(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ApiError::PathDoesNotExist(path.to_string_lossy().into_owned())
        } else {
            ApiError::InvalidPath
        }
    })?;

    if !canonical.starts_with(&canonical_root) {
        return Err(ApiError::InvalidPath);
    }

    Ok(canonical)
}

#[derive(PartialEq, Debug)]
pub enum FileSchemes {
    File,
    ExtractedFile,
}

/// Resolves a scheme-prefixed path to a real filesystem path.
/// This function performs scheme resolution only. Callers are responsible
/// for enforcing any access bounds appropriate to their context.
pub fn resolve_scheme_path(path: &str) -> Result<(FileSchemes, PathBuf), ApiError> {
    use super::storage::TMP_PATH;

    let scheme = path.split("://").next().ok_or(ApiError::InvalidPath)?;

    match scheme {
        "file" => Ok((FileSchemes::File, PathBuf::from(&path[7..]))),
        "extracted-file" => Ok((FileSchemes::ExtractedFile, TMP_PATH.join(&path[17..]))),
        _ => Err(ApiError::InvalidPath),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::storage::TMP_PATH;
    use std::fs;
    use std::os::unix::fs::symlink;
    use tempfile::tempdir;

    #[test]
    fn test_validate_path_within_bounds_valid_path() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();

        let result = validate_path_within_bounds(&test_file, temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_within_bounds_traversal() {
        let temp_dir = tempdir().unwrap();
        let outside_dir = tempdir().unwrap();
        let test_file = outside_dir.path().join("outside.txt");
        fs::write(&test_file, "test").unwrap();

        let result = validate_path_within_bounds(&test_file, temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_within_bounds_nonexistent() {
        let temp_dir = tempdir().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent.txt");

        let result = validate_path_within_bounds(&nonexistent, temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_scheme_path_file() {
        let path = "file:///test.txt";
        let (scheme, path) = resolve_scheme_path(path).unwrap();

        assert_eq!(path, PathBuf::from("/test.txt"));
        assert_eq!(scheme, FileSchemes::File)
    }

    #[test]
    fn test_resolve_scheme_path_extracted_file() {
        let path = "extracted-file://cover.jpg";
        let (scheme, path) = resolve_scheme_path(path).unwrap();

        assert_eq!(path, TMP_PATH.join("cover.jpg"));
        assert_eq!(scheme, FileSchemes::ExtractedFile);
    }

    #[test]
    fn test_resolve_scheme_path_invalid_scheme() {
        let path = "invalid://path";
        let result = resolve_scheme_path(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_within_bounds_symlink() {
        // Create a symlink that points outside the allowed directory
        // This tests that canonicalization catches symlink-based escapes
        let temp_dir = tempdir().unwrap();
        let allowed = temp_dir.path().join("allowed");
        let outside = temp_dir.path().join("outside");

        fs::create_dir(&allowed).unwrap();
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("secret.txt"), "secret").unwrap();

        // Create symlink from allowed to outside
        let link = allowed.join("link.txt");
        if symlink(outside.join("secret.txt"), &link).is_ok() {
            // Symlink was created, test that canonicalization catches it
            let result = validate_path_within_bounds(&link, allowed.as_path());
            assert!(result.is_err(), "Symlink traversal should be rejected");
        } else {
            // Symlinks not supported on this platform (e.g., Windows)
            println!("Skipping symlink test - not supported");
        }
    }

    #[test]
    fn test_validate_path_within_bounds_double_dot_valid() {
        // Path with .. that stays within bounds
        let temp_dir = tempdir().unwrap();
        let sub_dir = temp_dir.path().join("sub");
        fs::create_dir(&sub_dir).unwrap();

        let test_file = sub_dir.join("../test.txt");
        fs::write(&test_file, "test").unwrap();

        let result = validate_path_within_bounds(&test_file, temp_dir.path());
        assert!(result.is_ok(), "Valid path with .. should be accepted");
    }

    #[test]
    fn test_validate_path_within_bounds_double_dot_escape() {
        // Path with .. that escapes bounds
        let temp_dir = tempdir().unwrap();
        let _outside = tempdir().unwrap();

        let test_file = temp_dir.path().join("../outside.txt");
        fs::write(&test_file, "test").unwrap();

        let result = validate_path_within_bounds(&test_file, temp_dir.path());
        assert!(result.is_err(), "Path escape with .. should be rejected");
    }
}
