//! Minimum-version enforcement - removed from this build.
//!
//! The former module probed the release registry and could download and
//! install a newer binary to satisfy a configured version floor. Update
//! checks are removed from this build, so enforcement is a no-op: no
//! network I/O, no process spawned, no exit.

use crate::version::UpdateConfig;

/// Former minimum-version gate. Update checks removed from this build:
/// no-op - never probes a registry, never installs, never exits.
pub async fn enforce_minimum_version_or_exit(_update_config: &UpdateConfig) {
    tracing::debug!("minimum-version enforcement skipped: update checks disabled in this build");
}
