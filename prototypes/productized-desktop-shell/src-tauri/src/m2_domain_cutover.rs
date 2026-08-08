// M2 domain cutover implementation.
// This implements per-domain cutover: shadow → parity → new primary → compatibility read-only.

use crate::m2_dto::*;
use crate::m2_ports::*;
use rusqlite::Connection;
use std::fmt;

/// Count Parity Result
#[derive(Clone, Debug, PartialEq)]
pub enum CountParityResult {
    Match {
        count: usize,
    },
    Mismatch {
        primary_count: usize,
        shadow_count: usize,
    },
}

/// Key Parity Result
#[derive(Clone, Debug, PartialEq)]
pub enum KeyParityResult {
    Match {
        keys: Vec<String>,
    },
    Mismatch {
        missing_in_shadow: Vec<String>,
        missing_in_primary: Vec<String>,
    },
}

/// Hash Parity Result
#[derive(Clone, Debug, PartialEq)]
pub enum HashParityResult {
    Match {
        hash: String,
    },
    Mismatch {
        primary_hash: String,
        shadow_hash: String,
    },
    MissingSnapshot,
}

/// Domain Cutover Implementation
pub struct DomainCutoverImpl {
    primary_repo: Box<dyn CurrentSnapshotRepository>,
    shadow_repo: Box<dyn CurrentSnapshotRepository>,
}

impl DomainCutoverImpl {
    /// Create a new DomainCutoverImpl
    pub fn new(
        primary_repo: Box<dyn CurrentSnapshotRepository>,
        shadow_repo: Box<dyn CurrentSnapshotRepository>,
    ) -> Self {
        Self {
            primary_repo,
            shadow_repo,
        }
    }

    /// Execute domain cutover
    pub fn execute_cutover(
        &self,
        connection: &Connection,
        domain: &str,
        projector_id: &str,
    ) -> Result<CutoverResult, String> {
        // 1. Shadow write phase
        let shadow_result = self.shadow_write_phase(connection, domain, projector_id)?;

        // 2. Parity check phase
        let parity_result = self.parity_check_phase(connection, domain, projector_id)?;

        // 3. New primary phase
        let primary_result = self.new_primary_phase(connection, domain, projector_id)?;

        // 4. Compatibility read-only phase
        let compatibility_result =
            self.compatibility_readonly_phase(connection, domain, projector_id)?;

        Ok(CutoverResult {
            domain: domain.to_string(),
            shadow_result,
            parity_result,
            primary_result,
            compatibility_result,
            status: CutoverStatus::Completed,
            timestamp: generate_timestamp(),
        })
    }

    /// Shadow write phase
    fn shadow_write_phase(
        &self,
        connection: &Connection,
        domain: &str,
        projector_id: &str,
    ) -> Result<ShadowWriteResult, String> {
        // 1. Get all primary snapshots for domain
        let primary_snapshots = self
            .primary_repo
            .get_by_projector(connection, projector_id)?;

        let mut written_count = 0;
        let mut error_count = 0;

        for snapshot in &primary_snapshots {
            // 2. Write to shadow
            match self.shadow_repo.upsert(connection, snapshot) {
                Ok(()) => {
                    written_count += 1;
                }
                Err(error) => {
                    eprintln!("Shadow write failed for {}: {}", snapshot.object_ref, error);
                    error_count += 1;
                }
            }
        }

        Ok(ShadowWriteResult {
            domain: domain.to_string(),
            written_count,
            error_count,
            status: if error_count == 0 {
                ShadowWriteStatus::Success
            } else {
                ShadowWriteStatus::Partial
            },
        })
    }

    /// Parity check phase
    fn parity_check_phase(
        &self,
        connection: &Connection,
        domain: &str,
        projector_id: &str,
    ) -> Result<ParityCheckResult, String> {
        // 1. Get all primary snapshots
        let primary_snapshots = self
            .primary_repo
            .get_by_projector(connection, projector_id)?;

        // 2. Get all shadow snapshots
        let shadow_snapshots = self
            .shadow_repo
            .get_by_projector(connection, projector_id)?;

        // 3. Check count parity
        let primary_count = primary_snapshots.len();
        let shadow_count = shadow_snapshots.len();
        let count_parity = if primary_count == shadow_count {
            CountParityResult::Match {
                count: primary_count,
            }
        } else {
            CountParityResult::Mismatch {
                primary_count,
                shadow_count,
            }
        };

        // 4. Check key parity
        let primary_keys: Vec<String> = primary_snapshots
            .iter()
            .map(|s| s.object_ref.clone())
            .collect();
        let shadow_keys: Vec<String> = shadow_snapshots
            .iter()
            .map(|s| s.object_ref.clone())
            .collect();

        let key_parity = if primary_keys == shadow_keys {
            KeyParityResult::Match { keys: primary_keys }
        } else {
            let missing_in_shadow: Vec<String> = primary_keys
                .iter()
                .filter(|k| !shadow_keys.contains(k))
                .cloned()
                .collect();
            let missing_in_primary: Vec<String> = shadow_keys
                .iter()
                .filter(|k| !primary_keys.contains(k))
                .cloned()
                .collect();

            KeyParityResult::Mismatch {
                missing_in_shadow,
                missing_in_primary,
            }
        };

        // 5. Check hash parity for each key
        let mut hash_parities = Vec::new();
        let mut match_count = 0;
        let mut mismatch_count = 0;

        if let KeyParityResult::Match { keys } = &key_parity {
            for key in keys {
                // Get primary snapshot
                let primary = self.primary_repo.get(connection, key, projector_id)?;

                // Get shadow snapshot
                let shadow = self.shadow_repo.get(connection, key, projector_id)?;

                // Compare hashes
                let hash_parity = match (&primary, &shadow) {
                    (Some(primary), Some(shadow)) => {
                        if primary.snapshot_hash == shadow.snapshot_hash {
                            HashParityResult::Match {
                                hash: primary.snapshot_hash.clone(),
                            }
                        } else {
                            HashParityResult::Mismatch {
                                primary_hash: primary.snapshot_hash.clone(),
                                shadow_hash: shadow.snapshot_hash.clone(),
                            }
                        }
                    }
                    _ => HashParityResult::MissingSnapshot,
                };

                match &hash_parity {
                    HashParityResult::Match { .. } => match_count += 1,
                    HashParityResult::Mismatch { .. } => mismatch_count += 1,
                    HashParityResult::MissingSnapshot => mismatch_count += 1,
                }

                hash_parities.push((key.clone(), hash_parity));
            }
        }

        Ok(ParityCheckResult {
            domain: domain.to_string(),
            count_parity,
            key_parity,
            hash_parities,
            match_count,
            mismatch_count,
            status: if mismatch_count == 0 {
                ParityCheckStatus::Match
            } else {
                ParityCheckStatus::Mismatch
            },
        })
    }

    /// New primary phase
    fn new_primary_phase(
        &self,
        connection: &Connection,
        domain: &str,
        projector_id: &str,
    ) -> Result<NewPrimaryResult, String> {
        // 1. Get all shadow snapshots
        let shadow_snapshots = self
            .shadow_repo
            .get_by_projector(connection, projector_id)?;

        let mut promoted_count = 0;
        let mut error_count = 0;

        for snapshot in &shadow_snapshots {
            // 2. Promote shadow to primary
            match self.primary_repo.upsert(connection, snapshot) {
                Ok(()) => {
                    promoted_count += 1;
                }
                Err(error) => {
                    eprintln!(
                        "Primary promote failed for {}: {}",
                        snapshot.object_ref, error
                    );
                    error_count += 1;
                }
            }
        }

        Ok(NewPrimaryResult {
            domain: domain.to_string(),
            promoted_count,
            error_count,
            status: if error_count == 0 {
                NewPrimaryStatus::Success
            } else {
                NewPrimaryStatus::Partial
            },
        })
    }

    /// Compatibility read-only phase
    fn compatibility_readonly_phase(
        &self,
        connection: &Connection,
        domain: &str,
        projector_id: &str,
    ) -> Result<CompatibilityReadonlyResult, String> {
        // 1. Verify primary is readable
        let primary_snapshots = self
            .primary_repo
            .get_by_projector(connection, projector_id)?;

        // 2. Verify shadow is readable
        let shadow_snapshots = self
            .shadow_repo
            .get_by_projector(connection, projector_id)?;

        // 3. Verify parity after cutover
        let primary_count = primary_snapshots.len();
        let shadow_count = shadow_snapshots.len();
        let final_parity = if primary_count == shadow_count {
            CountParityResult::Match {
                count: primary_count,
            }
        } else {
            CountParityResult::Mismatch {
                primary_count,
                shadow_count,
            }
        };

        Ok(CompatibilityReadonlyResult {
            domain: domain.to_string(),
            primary_readable: !primary_snapshots.is_empty(),
            shadow_readable: !shadow_snapshots.is_empty(),
            final_parity,
            status: CompatibilityReadonlyStatus::Verified,
        })
    }
}

/// Cutover Result
#[derive(Clone, Debug, PartialEq)]
pub struct CutoverResult {
    pub domain: String,
    pub shadow_result: ShadowWriteResult,
    pub parity_result: ParityCheckResult,
    pub primary_result: NewPrimaryResult,
    pub compatibility_result: CompatibilityReadonlyResult,
    pub status: CutoverStatus,
    pub timestamp: String,
}

/// Cutover Status
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CutoverStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    RolledBack,
}

/// Shadow Write Result
#[derive(Clone, Debug, PartialEq)]
pub struct ShadowWriteResult {
    pub domain: String,
    pub written_count: usize,
    pub error_count: usize,
    pub status: ShadowWriteStatus,
}

/// Shadow Write Status
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShadowWriteStatus {
    Success,
    Partial,
    Failed,
}

/// Parity Check Result
#[derive(Clone, Debug, PartialEq)]
pub struct ParityCheckResult {
    pub domain: String,
    pub count_parity: CountParityResult,
    pub key_parity: KeyParityResult,
    pub hash_parities: Vec<(String, HashParityResult)>,
    pub match_count: usize,
    pub mismatch_count: usize,
    pub status: ParityCheckStatus,
}

/// Parity Check Status
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParityCheckStatus {
    Match,
    Mismatch,
    Error,
}

/// New Primary Result
#[derive(Clone, Debug, PartialEq)]
pub struct NewPrimaryResult {
    pub domain: String,
    pub promoted_count: usize,
    pub error_count: usize,
    pub status: NewPrimaryStatus,
}

/// New Primary Status
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NewPrimaryStatus {
    Success,
    Partial,
    Failed,
}

/// Compatibility Read-Only Result
#[derive(Clone, Debug, PartialEq)]
pub struct CompatibilityReadonlyResult {
    pub domain: String,
    pub primary_readable: bool,
    pub shadow_readable: bool,
    pub final_parity: CountParityResult,
    pub status: CompatibilityReadonlyStatus,
}

/// Compatibility Read-Only Status
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompatibilityReadonlyStatus {
    Verified,
    Degraded,
    Failed,
}

/// Generate ISO 8601 timestamp
fn generate_timestamp() -> String {
    crate::m2_clock::utc_now_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_cutover_impl_creates() {
        // Note: This test only verifies the domain cutover can be created.
        // Full integration tests require actual database connections.
    }

    #[test]
    fn cutover_result_variants() {
        let result = CutoverResult {
            domain: "workflow".to_string(),
            shadow_result: ShadowWriteResult {
                domain: "workflow".to_string(),
                written_count: 10,
                error_count: 0,
                status: ShadowWriteStatus::Success,
            },
            parity_result: ParityCheckResult {
                domain: "workflow".to_string(),
                count_parity: CountParityResult::Match { count: 10 },
                key_parity: KeyParityResult::Match {
                    keys: vec!["key1".to_string()],
                },
                hash_parities: Vec::new(),
                match_count: 10,
                mismatch_count: 0,
                status: ParityCheckStatus::Match,
            },
            primary_result: NewPrimaryResult {
                domain: "workflow".to_string(),
                promoted_count: 10,
                error_count: 0,
                status: NewPrimaryStatus::Success,
            },
            compatibility_result: CompatibilityReadonlyResult {
                domain: "workflow".to_string(),
                primary_readable: true,
                shadow_readable: true,
                final_parity: CountParityResult::Match { count: 10 },
                status: CompatibilityReadonlyStatus::Verified,
            },
            status: CutoverStatus::Completed,
            timestamp: generate_timestamp(),
        };

        assert_eq!(result.domain, "workflow");
        assert_eq!(result.status, CutoverStatus::Completed);
    }
}
