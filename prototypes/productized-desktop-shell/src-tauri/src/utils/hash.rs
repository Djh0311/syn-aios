use sha2::{Digest, Sha256};

pub(crate) const WORKBENCH_SOURCE_AGGREGATE_HASH_ALGORITHM: &str =
    "workbench_source_aggregate_hash.v1:preflight_path_ref_file_hash_classification_concat";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WorkbenchSourceAggregateHashEntry<'a> {
    pub(crate) path_ref: &'a str,
    pub(crate) file_hash: Option<&'a str>,
    pub(crate) classification: &'a str,
}

pub(crate) fn sha256_hex(value: &str) -> String {
    sha256_hex_bytes(value.as_bytes())
}

pub(crate) fn sha256_hex_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub(crate) fn short_hash(value: &str) -> String {
    short_hash_len(value, 16)
}

pub(crate) fn short_hash12(value: &str) -> String {
    short_hash_len(value, 12)
}

pub(crate) fn short_hash_len(value: &str, len: usize) -> String {
    sha256_hex(value).chars().take(len).collect()
}

pub(crate) fn workbench_source_aggregate_hash<'a>(
    entries: impl IntoIterator<Item = WorkbenchSourceAggregateHashEntry<'a>>,
) -> String {
    let mut input = String::new();
    for entry in entries {
        input.push_str(entry.path_ref);
        if let Some(hash) = entry.file_hash {
            input.push_str(hash);
        }
        input.push_str(entry.classification);
    }
    sha256_hex_bytes(input.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_helpers_preserve_existing_lengths() {
        let value = "workbench-hash-helper";
        let full = sha256_hex(value);

        assert_eq!(full.len(), 64);
        assert_eq!(sha256_hex_bytes(value.as_bytes()), full);
        assert_eq!(short_hash(value), full.chars().take(16).collect::<String>());
        assert_eq!(
            short_hash12(value),
            full.chars().take(12).collect::<String>()
        );
    }

    #[test]
    fn workbench_source_aggregate_hash_preserves_preflight_v1_contract() {
        let entries = [
            WorkbenchSourceAggregateHashEntry {
                path_ref: "plan-authorizations.v1.json",
                file_hash: Some("6962e4781f49246525d4cde37d3133924a66faa12b8aab90db106c3c9f401b0e"),
                classification: "accepted",
            },
            WorkbenchSourceAggregateHashEntry {
                path_ref: "workflow-state.v0.json",
                file_hash: Some("4bd5434fdca9e82c8fafc42989e1a267ed7d677bfe2972273fb3afaa26829972"),
                classification: "accepted",
            },
        ];

        assert_eq!(
            WORKBENCH_SOURCE_AGGREGATE_HASH_ALGORITHM,
            "workbench_source_aggregate_hash.v1:preflight_path_ref_file_hash_classification_concat"
        );
        assert_eq!(
            workbench_source_aggregate_hash(entries),
            "31cdea623d928ea2dc13d0a02eaefd23f2df1a27f454d5d7ea17d51fe3b4b801"
        );
    }
}
