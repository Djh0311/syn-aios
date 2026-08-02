// SYN-FND-002: Object ID path guard.
//
// All canvas / run / node / template IDs that enter PathBuf::join MUST pass
// through a validator from this module before being used in path construction.
//
// Design: single-segment IDs only (no `/` allowed). The validator rejects
// traversal, encoding tricks, and control characters at the gate — before any
// filesystem mutation.
//
// Contract: docs/contracts/identity-scope-v1.md §3.2 canvas path section.
// Evidence level: STATIC_OPENING_ONLY → will be upgraded after focused tests.

use std::path::{Path, PathBuf};

/// Maximum allowed length for a single object ID segment.
const MAX_OBJECT_ID_LEN: usize = 256;

/// Validated, safe-for-path-construction object ID.
///
/// Invariants:
/// - Non-empty
/// - No `/` or `\` (single segment only)
/// - Not `.` or `..`
/// - No absolute path prefix (POSIX `/` or Windows `C:\`)
/// - No percent-encoding (`%xx`)
/// - No null bytes or control characters (U+0000..U+001F, U+007F..U+009F)
/// - No Unicode normalization variants that could confuse path resolution
/// - Length within `MAX_OBJECT_ID_LEN`
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ValidatedObjectId(String);

impl ValidatedObjectId {
    /// Validate a raw string and produce a `ValidatedObjectId` if it passes all checks.
    ///
    /// Returns `Err(reason)` on rejection — the reason is safe to log (no secrets).
    pub fn parse(raw: &str) -> Result<Self, String> {
        // 1. Empty check
        if raw.is_empty() {
            return Err("path_guard_rejected: object ID 不能为空".to_string());
        }

        // 2. Length check
        if raw.len() > MAX_OBJECT_ID_LEN {
            return Err(format!(
                "path_guard_rejected: object ID 长度 {} 超过上限 {}",
                raw.len(),
                MAX_OBJECT_ID_LEN
            ));
        }

        // 3. Null byte check (must be first — controls can slip past other checks)
        if raw.contains('\0') {
            return Err("path_guard_rejected: object ID 含空字节".to_string());
        }

        // 4. Control character check (C0 controls + DEL + C1 controls)
        if raw.chars().any(|c| {
            let u = c as u32;
            (0x00..=0x1F).contains(&u) || u == 0x7F || (0x80..=0x9F).contains(&u)
        }) {
            return Err("path_guard_rejected: object ID 含控制字符".to_string());
        }

        // 5. Path separator check — single segment only
        if raw.contains('/') || raw.contains('\\') {
            return Err("path_guard_rejected: object ID 含路径分隔符".to_string());
        }

        // 6. Dot-segment check
        if raw == "." || raw == ".." {
            return Err("path_guard_rejected: object ID 是 . 或 ..".to_string());
        }

        // 7. Absolute path check (POSIX)
        if raw.starts_with('/') {
            return Err("path_guard_rejected: object ID 是绝对路径".to_string());
        }

        // 8. Absolute path check (Windows drive letter)
        if raw.len() >= 2
            && raw.as_bytes()[0].is_ascii_alphabetic()
            && raw.as_bytes()[1] == b':'
        {
            return Err("path_guard_rejected: object ID 含 Windows 盘符".to_string());
        }

        // 9. Percent-encoding check (could decode to traversal)
        if raw.contains('%') {
            return Err("path_guard_rejected: object ID 含百分号编码".to_string());
        }

        // 10. Backslash is already caught by #5, but double-check for UNC paths
        if raw.starts_with("\\\\") {
            return Err("path_guard_rejected: object ID 是 UNC 路径".to_string());
        }

        // 11. Unicode normalization check — reject NFKC/NFKD normalization variants
        //     that could confuse path resolution (e.g., fullwidth `/` U+FF0F)
        if raw.chars().any(|c| {
            // Fullwidth solidus, fullwidth reverse solidus, fraction slash,
            // division slash, and other confusable path characters
            matches!(
                c,
                '\u{FF0F}' // fullwidth solidus `/`
                | '\u{FF3C}' // fullwidth reverse solidus `\`
                | '\u{2044}' // fraction slash `⁄`
                | '\u{2215}' // division slash `∕`
                | '\u{29F8}' // big solidus `⧸`
                | '\u{29F9}' // big reverse solidus `⧹`
            )
        }) {
            return Err("path_guard_rejected: object ID 含 Unicode 路径混淆字符".to_string());
        }

        Ok(Self(raw.to_string()))
    }

    /// Get the validated ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the validated ID as a `Path` reference.
    pub fn as_path(&self) -> &Path {
        Path::new(&self.0)
    }

    /// Consume and return the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for ValidatedObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Validate that a resolved path stays within the allowed root directory.
///
/// This is the second defense layer: after ID validation, we canonicalize
/// the resolved path and check it's still under `allowed_root`.
///
/// Returns the canonical path on success, or an error if traversal is detected.
pub fn validate_resolved_path_within_root(
    resolved: &Path,
    allowed_root: &Path,
) -> Result<PathBuf, String> {
    let canonical = resolved
        .canonicalize()
        .map_err(|e| format!("path_guard_rejected: 无法规范化路径: {e}"))?;

    let root_canonical = allowed_root
        .canonicalize()
        .map_err(|e| format!("path_guard_rejected: 无法规范化允许根: {e}"))?;

    if !canonical.starts_with(&root_canonical) {
        return Err(format!(
            "path_guard_rejected: 路径 {} 逃逸出允许根 {}",
            canonical.display(),
            root_canonical.display()
        ));
    }

    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Positive cases ----

    #[test]
    fn valid_simple_id() {
        assert!(ValidatedObjectId::parse("abc123").is_ok());
    }

    #[test]
    fn valid_uuid_like() {
        assert!(ValidatedObjectId::parse("019fbdb1-b50f-71d3-a482-5f96a21595a8").is_ok());
    }

    #[test]
    fn valid_with_underscores_and_hyphens() {
        assert!(ValidatedObjectId::parse("canvas_v1-run-001").is_ok());
    }

    #[test]
    fn valid_with_dots_in_name() {
        // Dots inside the name are fine (e.g., "file.name.json" as an ID)
        // The only forbidden dots are standalone `.` and `..`
        assert!(ValidatedObjectId::parse("my.canvas.v1").is_ok());
    }

    #[test]
    fn valid_max_length() {
        let id = "a".repeat(MAX_OBJECT_ID_LEN);
        assert!(ValidatedObjectId::parse(&id).is_ok());
    }

    // ---- Negative cases: empty / length ----

    #[test]
    fn reject_empty() {
        assert!(ValidatedObjectId::parse("").is_err());
    }

    #[test]
    fn reject_too_long() {
        let id = "a".repeat(MAX_OBJECT_ID_LEN + 1);
        assert!(ValidatedObjectId::parse(&id).is_err());
    }

    // ---- Negative cases: traversal ----

    #[test]
    fn reject_single_dot() {
        assert!(ValidatedObjectId::parse(".").is_err());
    }

    #[test]
    fn reject_double_dot() {
        assert!(ValidatedObjectId::parse("..").is_err());
    }

    #[test]
    fn reject_dot_slash() {
        assert!(ValidatedObjectId::parse("./foo").is_err());
    }

    #[test]
    fn reject_double_dot_slash() {
        assert!(ValidatedObjectId::parse("../etc/passwd").is_err());
    }

    #[test]
    fn reject_trailing_dot_dot() {
        assert!(ValidatedObjectId::parse("foo/..").is_err());
    }

    // ---- Negative cases: path separators ----

    #[test]
    fn reject_forward_slash() {
        assert!(ValidatedObjectId::parse("foo/bar").is_err());
    }

    #[test]
    fn reject_backslash() {
        assert!(ValidatedObjectId::parse("foo\\bar").is_err());
    }

    #[test]
    fn reject_leading_slash() {
        assert!(ValidatedObjectId::parse("/etc/passwd").is_err());
    }

    #[test]
    fn reject_trailing_slash() {
        assert!(ValidatedObjectId::parse("foo/").is_err());
    }

    // ---- Negative cases: absolute paths ----

    #[test]
    fn reject_posix_absolute() {
        assert!(ValidatedObjectId::parse("/tmp/test").is_err());
    }

    #[test]
    fn reject_windows_drive() {
        assert!(ValidatedObjectId::parse("C:\\Windows").is_err());
    }

    #[test]
    fn reject_unc_path() {
        assert!(ValidatedObjectId::parse("\\\\server\\share").is_err());
    }

    // ---- Negative cases: encoding tricks ----

    #[test]
    fn reject_percent_encoded() {
        assert!(ValidatedObjectId::parse("foo%2Fbar").is_err());
    }

    #[test]
    fn reject_percent_dot_dot() {
        assert!(ValidatedObjectId::parse("%2e%2e").is_err());
    }

    #[test]
    fn reject_percent_encoded_null() {
        assert!(ValidatedObjectId::parse("foo%00bar").is_err());
    }

    // ---- Negative cases: control characters ----

    #[test]
    fn reject_null_byte() {
        assert!(ValidatedObjectId::parse("foo\0bar").is_err());
    }

    #[test]
    fn reject_tab() {
        assert!(ValidatedObjectId::parse("foo\tbar").is_err());
    }

    #[test]
    fn reject_newline() {
        assert!(ValidatedObjectId::parse("foo\nbar").is_err());
    }

    #[test]
    fn reject_del() {
        assert!(ValidatedObjectId::parse("foo\x7Fbar").is_err());
    }

    // ---- Negative cases: Unicode confusables ----

    #[test]
    fn reject_fullwidth_solidus() {
        assert!(ValidatedObjectId::parse("foo\u{FF0F}bar").is_err());
    }

    #[test]
    fn reject_fullwidth_backslash() {
        assert!(ValidatedObjectId::parse("foo\u{FF3C}bar").is_err());
    }

    #[test]
    fn reject_fraction_slash() {
        assert!(ValidatedObjectId::parse("foo\u{2044}bar").is_err());
    }

    // ---- Display ----

    #[test]
    fn display_returns_id() {
        let id = ValidatedObjectId::parse("test-123").unwrap();
        assert_eq!(format!("{id}"), "test-123");
    }

    // ---- validate_resolved_path_within_root ----

    #[test]
    fn resolved_path_within_root() {
        let root = std::env::temp_dir().join("syn-fnd-002-test-root");
        let _ = std::fs::create_dir_all(&root);
        // Create a child file so canonicalize works
        let child_file = root.join("child.txt");
        let _ = std::fs::write(&child_file, "test");
        let result = validate_resolved_path_within_root(&child_file, &root);
        assert!(result.is_ok());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn resolved_path_escape_rejected() {
        let root = std::env::temp_dir().join("syn-fnd-002-test-root-escape");
        let _ = std::fs::create_dir_all(&root);
        // Create a file outside the root
        let outside = std::env::temp_dir().join("syn-fnd-002-outside.txt");
        let _ = std::fs::write(&outside, "test");
        let result = validate_resolved_path_within_root(&outside, &root);
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_file(&outside);
    }
}
