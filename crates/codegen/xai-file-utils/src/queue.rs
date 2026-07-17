//! Upload queue - cloud upload removed from this build.
//!
//! The former spill-to-disk queue captured artifacts to temp files and
//! uploaded them to cloud storage in the background. This build ships
//! without any cloud-upload capability: the public API surface is
//! preserved so callers compile, but nothing is ever written to the
//! network. Enqueue calls are no-ops (or report a failure where the
//! caller awaits a completion), and the only remaining behavior is the
//! local janitor that removes stale `upload_queue/` leftovers from
//! previous installs.

use crate::TraceExportConfig;
use crate::storage_client::Auth401AttributionCallback;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::{Notify, oneshot};
use xai_grok_auth::AuthCredentialProvider;

const UPLOAD_REMOVED: &str = "cloud upload removed from this build";

/// Resolves upload credentials/config. Retained for API compatibility so
/// implementors keep compiling; never consulted in this build (no uploads
/// happen).
pub trait TraceExportSource: Send + Sync {
    fn resolve(&self) -> TraceExportConfig;
    /// Async variant. Override to drive auth refresh; default delegates to sync.
    fn resolve_async(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = TraceExportConfig> + Send + '_>> {
        Box::pin(std::future::ready(self.resolve()))
    }
    /// Retained for API compatibility; unused (no uploads happen).
    fn proxy_attribution(&self) -> Option<Arc<dyn Auth401AttributionCallback>> {
        None
    }
    /// Retained for API compatibility; unused (no uploads happen).
    fn proxy_credentials(&self) -> Option<Arc<dyn AuthCredentialProvider>> {
        None
    }
    /// Retained for API compatibility; unused (no uploads happen).
    fn proxy_http_client(&self) -> Option<reqwest::Client> {
        None
    }
    /// Retained for API compatibility; unused (no uploads happen).
    fn wait_for_auth_recovery(
        &self,
        failed_bearer: Option<&str>,
        timeout: Duration,
    ) -> Option<std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>>> {
        let _ = (failed_bearer, timeout);
        None
    }
    /// Retained for API compatibility; unused (no uploads happen).
    fn has_usable_credential(&self) -> bool {
        true
    }
}

/// Default max age for upload queue leftovers; used by the startup orphan
/// cleanup.
pub const DEFAULT_MAX_AGE: Duration = Duration::from_secs(2 * 60 * 60);

/// Retry policy shape. Retained for API compatibility; unused in this build
/// (no uploads happen, so nothing retries).
#[derive(Clone, Debug)]
pub struct UploadRetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub max_age: Duration,
    pub auth_park_probe_interval: Duration,
}

pub const DEFAULT_AUTH_PARK_PROBE_INTERVAL: Duration = Duration::from_secs(300);

impl Default for UploadRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 10,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(120),
            multiplier: 2.0,
            max_age: DEFAULT_MAX_AGE,
            auth_park_probe_interval: DEFAULT_AUTH_PARK_PROBE_INTERVAL,
        }
    }
}

/// Schema version stamped on every [`QueueItemSidecar`].
pub const QUEUE_ITEM_SIDECAR_SCHEMA_VERSION: u32 = 1;

/// Sidecar manifest formerly written next to queue temp files. Retained so
/// the restart-recovery scanner keeps compiling (it now only finds and
/// removes stale pairs; nothing is re-uploaded).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct QueueItemSidecar {
    #[serde(default = "default_sidecar_schema_version")]
    pub schema_version: u32,
    pub session_id: String,
    pub turn_number: u64,
    pub gcs_path: String,
    pub content_type: String,
    pub artifact_name: String,
    pub enqueued_at: String,
    pub sha256: String,
}

fn default_sidecar_schema_version() -> u32 {
    QUEUE_ITEM_SIDECAR_SCHEMA_VERSION
}

/// Completion info shape. Retained for API compatibility; in this build no
/// upload ever completes, so this is never constructed with a real URL.
#[derive(Debug)]
pub struct UploadCompletion {
    pub gcs_url: String,
    pub compression: crate::BlobCompression,
    pub original_size: u64,
    pub stored_size: u64,
}

/// Result of enqueueing a file. The completion receiver resolves immediately
/// with an error in this build.
pub struct EnqueueResult {
    pub completion_rx: oneshot::Receiver<anyhow::Result<UploadCompletion>>,
    pub original_size: u64,
}

/// Queue statistics. Retained for API compatibility; all counters stay at
/// zero in this build (nothing is enqueued or uploaded).
pub struct UploadQueueStats {
    pub pending: AtomicU64,
    pub pending_bytes: AtomicU64,
    pub inflight: AtomicU64,
    pub enqueued: AtomicU64,
    pub deduplicated: AtomicU64,
    pub uploaded: AtomicU64,
    pub failed: AtomicU64,
    pub circuit_breaker_trips: AtomicU64,
    pub circuit_breaker_active: AtomicBool,
    pub enqueue_fallbacks: AtomicU64,
    pub leaked_temp_files: AtomicU64,
    pub reference_stale: AtomicU64,
    pub auth_parked: AtomicU64,
    pub cleanup_orphan_mismatched: AtomicU64,
    transition_notify: OnceLock<Arc<Notify>>,
}

impl Default for UploadQueueStats {
    fn default() -> Self {
        Self::new()
    }
}

impl UploadQueueStats {
    pub fn new() -> Self {
        Self {
            pending: AtomicU64::new(0),
            pending_bytes: AtomicU64::new(0),
            inflight: AtomicU64::new(0),
            enqueued: AtomicU64::new(0),
            deduplicated: AtomicU64::new(0),
            uploaded: AtomicU64::new(0),
            failed: AtomicU64::new(0),
            circuit_breaker_trips: AtomicU64::new(0),
            circuit_breaker_active: AtomicBool::new(false),
            enqueue_fallbacks: AtomicU64::new(0),
            leaked_temp_files: AtomicU64::new(0),
            reference_stale: AtomicU64::new(0),
            auth_parked: AtomicU64::new(0),
            cleanup_orphan_mismatched: AtomicU64::new(0),
            transition_notify: OnceLock::new(),
        }
    }

    /// Wire an external transition listener. Set once; a second call is a
    /// no-op. Never pinged in this build (counters never change).
    pub fn set_transition_notify(&self, notify: Arc<Notify>) {
        let _ = self.transition_notify.set(notify);
    }
}

/// Remove `path`; on non-`NotFound` failure, warn and bump
/// `leaked_temp_files` (when `stats` is `Some`).
pub fn try_remove_temp(path: &Path, stats: Option<&UploadQueueStats>) {
    if let Err(e) = std::fs::remove_file(path)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(
            path = % path.display(), error = % e,
            "Failed to remove upload-queue temp file; leaked"
        );
        if let Some(s) = stats {
            s.leaked_temp_files.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Handle for the former background upload queue.
///
/// Cloud upload removed from this build: no worker is spawned, nothing is
/// spilled to disk, and no enqueue call performs (or schedules) network I/O.
#[derive(Clone)]
pub struct UploadQueue {
    queue_dir: PathBuf,
    #[allow(dead_code)]
    resolver: Arc<dyn TraceExportSource>,
    stats: Arc<UploadQueueStats>,
    pub client_version: Option<String>,
    #[allow(dead_code)]
    max_queue_bytes: u64,
}

/// Marker error for [`UploadQueue::enqueue_blocking`]. Retained because
/// callers downcast to it.
#[derive(Debug)]
pub struct QueueClosed;
impl std::fmt::Display for QueueClosed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("upload queue worker is shut down")
    }
}
impl std::error::Error for QueueClosed {}

/// Structured outcome of [`UploadQueue::enqueue_bytes_blocking`]. In this
/// build every attempt reports [`EnqueueOutcome::Failed`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnqueueOutcome {
    Enqueued,
    FellBackToInline,
    Failed { reason: String },
    Deduplicated,
}

impl UploadQueue {
    /// Create the queue handle. No background worker is spawned in this build.
    pub fn spawn(
        grok_home: &Path,
        resolver: Arc<dyn TraceExportSource>,
        retry_policy: UploadRetryPolicy,
    ) -> Self {
        Self::spawn_with_concurrency(grok_home, resolver, retry_policy, 0)
    }

    /// Create the queue handle. No background worker is spawned in this build;
    /// `max_concurrent` is ignored.
    pub fn spawn_with_concurrency(
        grok_home: &Path,
        resolver: Arc<dyn TraceExportSource>,
        _retry_policy: UploadRetryPolicy,
        _max_concurrent: usize,
    ) -> Self {
        Self {
            queue_dir: grok_home.join("upload_queue"),
            resolver,
            stats: Arc::new(UploadQueueStats::new()),
            client_version: None,
            max_queue_bytes: 0,
        }
    }

    /// Retained for API compatibility; the version is never transmitted.
    pub fn with_client_version(mut self, version: impl Into<String>) -> Self {
        self.client_version = Some(version.into());
        self
    }

    /// Retained for API compatibility; there is no disk budget because
    /// nothing is spilled.
    pub fn with_max_queue_bytes(mut self, max_bytes: u64) -> Self {
        self.max_queue_bytes = max_bytes;
        self
    }

    /// Cloud upload removed from this build: no-op success, sends nothing.
    pub async fn enqueue(
        &self,
        _content: &[u8],
        gcs_path: &str,
        _content_type: &str,
        _artifact_name: &str,
        _session_id: &str,
        _turn_number: u64,
    ) -> anyhow::Result<()> {
        tracing::debug!(gcs_path, "upload queue disabled: {UPLOAD_REMOVED}");
        Ok(())
    }

    /// Cloud upload removed from this build: reports `Failed`, sends nothing.
    pub async fn enqueue_bytes_blocking(
        &self,
        _content: &[u8],
        _gcs_path: &str,
        _content_type: &str,
        _artifact_name: &str,
        _session_id: &str,
        _turn_number: u64,
    ) -> EnqueueOutcome {
        EnqueueOutcome::Failed {
            reason: UPLOAD_REMOVED.to_string(),
        }
    }

    /// Cloud upload removed from this build: reports `Failed`, sends nothing.
    /// The on-disk pair is left for the orphan janitor to age out.
    pub fn enqueue_recovered(
        &self,
        _temp_path: &Path,
        _sidecar_path: &Path,
        _sidecar: &QueueItemSidecar,
    ) -> EnqueueOutcome {
        EnqueueOutcome::Failed {
            reason: UPLOAD_REMOVED.to_string(),
        }
    }

    /// Cloud upload removed from this build: always returns an error, sends nothing.
    pub async fn enqueue_blocking(
        &self,
        _content: &[u8],
        gcs_path: &str,
        _content_type: &str,
        _artifact_name: &str,
        _session_id: &str,
        _turn_number: u64,
    ) -> anyhow::Result<String> {
        anyhow::bail!("{UPLOAD_REMOVED} (path: {gcs_path})")
    }

    /// Cloud upload removed from this build: no-op success, sends nothing.
    pub async fn enqueue_file(
        &self,
        _source_path: &Path,
        gcs_path: &str,
        _content_type: &str,
        _artifact_name: &str,
        _session_id: &str,
        _turn_number: u64,
    ) -> anyhow::Result<()> {
        tracing::debug!(gcs_path, "upload queue disabled: {UPLOAD_REMOVED}");
        Ok(())
    }

    /// Cloud upload removed from this build: the completion resolves
    /// immediately with an error; nothing is sent.
    #[allow(clippy::too_many_arguments)]
    pub async fn enqueue_file_blocking(
        &self,
        source_path: &Path,
        _gcs_path: &str,
        _content_type: &str,
        _artifact_name: &str,
        _session_id: &str,
        _turn_number: u64,
        _compress: bool,
    ) -> anyhow::Result<EnqueueResult> {
        let original_size = file_size(source_path);
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(Err(anyhow::anyhow!(UPLOAD_REMOVED)));
        Ok(EnqueueResult {
            completion_rx: rx,
            original_size,
        })
    }

    /// Cloud upload removed from this build: the completion resolves
    /// immediately with an error; nothing is snapshotted or sent.
    #[allow(clippy::too_many_arguments)]
    pub async fn enqueue_file_reference(
        &self,
        source_path: &Path,
        _expected_sha256: &str,
        _gcs_path: &str,
        _content_type: &str,
        _artifact_name: &str,
        _session_id: &str,
        _turn_number: u64,
    ) -> anyhow::Result<EnqueueResult> {
        let original_size = file_size(source_path);
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(Err(anyhow::anyhow!(UPLOAD_REMOVED)));
        Ok(EnqueueResult {
            completion_rx: rx,
            original_size,
        })
    }

    /// Nothing is ever pending in this build; returns 0 immediately.
    pub async fn wait_idle(&self, _timeout: Duration) -> usize {
        0
    }

    /// Nothing to drain in this build; returns 0 immediately.
    pub async fn drain(&self, _deadline: Duration) -> usize {
        0
    }

    /// Current queue statistics (all zeros in this build).
    pub fn stats(&self) -> &UploadQueueStats {
        &self.stats
    }

    /// Get a shared reference to the stats Arc for cross-component sharing.
    pub fn stats_arc(&self) -> Arc<UploadQueueStats> {
        self.stats.clone()
    }

    /// Clean up leftover `upload_queue/` entries from previous installs.
    /// Purely local disk hygiene; nothing is uploaded.
    pub fn cleanup_orphans(&self, max_age: Duration) {
        cleanup_queue_dir(&self.queue_dir, max_age, Some(&self.stats));
    }
}

pub const SIDECAR_SUFFIX: &str = ".meta.json";

/// Sidecar manifest path for a queue temp file: `<temp>.meta.json`.
pub fn sidecar_path_for(temp_path: &Path) -> PathBuf {
    let mut name = temp_path.as_os_str().to_owned();
    name.push(SIDECAR_SUFFIX);
    PathBuf::from(name)
}

/// Inverse of [`sidecar_path_for`]: the temp file a sidecar describes, or
/// `None` if `sidecar` does not carry the [`SIDECAR_SUFFIX`].
pub fn temp_path_for_sidecar(sidecar: &Path) -> Option<PathBuf> {
    let name = sidecar.file_name()?.to_str()?;
    let stem = name.strip_suffix(SIDECAR_SUFFIX)?;
    Some(sidecar.with_file_name(stem))
}

/// Get file size, returning 0 if the file doesn't exist.
fn file_size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

static LAST_ORPHANS_CLEANED: AtomicU64 = AtomicU64::new(0);

/// Number of orphaned entries cleaned by the last `cleanup_orphaned_uploads` call.
pub fn last_orphans_cleaned() -> u64 {
    LAST_ORPHANS_CLEANED.load(Ordering::Relaxed)
}

/// Clean up orphaned upload queue entries left behind by previous installs.
/// Purely local disk hygiene; nothing is uploaded.
pub fn cleanup_orphaned_uploads(grok_home: &Path, max_age: Duration) -> u64 {
    let cleaned = cleanup_queue_dir(&grok_home.join("upload_queue"), max_age, None);
    LAST_ORPHANS_CLEANED.store(cleaned, Ordering::Relaxed);
    cleaned
}

/// Sweep entries older than `max_age`. `scratch/` is treated specially:
/// recurse one level so per-session subdirs are aged independently.
fn cleanup_queue_dir(queue_dir: &Path, max_age: Duration, stats: Option<&UploadQueueStats>) -> u64 {
    let entries: Vec<std::fs::DirEntry> = match std::fs::read_dir(queue_dir) {
        Ok(e) => e.flatten().collect(),
        Err(_) => return 0,
    };
    let all_names: HashSet<std::ffi::OsString> = entries.iter().map(|e| e.file_name()).collect();
    let mut cleaned = 0u64;
    let mut cleaned_bytes = 0u64;
    for entry in &entries {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let path = entry.path();
        let name = entry.file_name();
        let is_scratch_root = metadata.is_dir() && name == "scratch";
        if is_scratch_root {
            let (sub_cleaned, sub_bytes) = cleanup_scratch_subdirs(&path, max_age);
            cleaned += sub_cleaned;
            cleaned_bytes += sub_bytes;
            continue;
        }
        let age = pair_age(&path, &name, &all_names).unwrap_or_else(|| {
            metadata
                .modified()
                .ok()
                .and_then(|m| m.elapsed().ok())
                .unwrap_or(Duration::MAX)
        });
        if age <= max_age {
            continue;
        }
        if metadata.is_dir() {
            let size = dir_size(&path).unwrap_or(0);
            if std::fs::remove_dir_all(&path).is_ok() {
                cleaned += 1;
                cleaned_bytes += size;
            }
        } else if std::fs::remove_file(&path).is_ok() {
            cleaned += 1;
            cleaned_bytes += metadata.len();
            if let Some(stats) = stats
                && is_mismatched_queue_file(&name, &all_names)
            {
                stats
                    .cleanup_orphan_mismatched
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    if cleaned > 0 {
        tracing::info!(
            cleaned, cleaned_bytes, dir = % queue_dir.display(),
            "Cleaned up orphaned upload queue entries from previous session"
        );
    }
    cleaned
}

/// Age of a queue file derived from its (or its companion's) sidecar
/// `enqueued_at`, or `None` when the file has no parseable sidecar.
fn pair_age(
    path: &Path,
    name: &std::ffi::OsStr,
    all_names: &HashSet<std::ffi::OsString>,
) -> Option<Duration> {
    let name_str = name.to_string_lossy();
    let sidecar_path = if name_str.ends_with(SIDECAR_SUFFIX) {
        path.to_path_buf()
    } else {
        let companion = format!("{name_str}{SIDECAR_SUFFIX}");
        if !all_names.contains(std::ffi::OsStr::new(companion.as_str())) {
            return None;
        }
        sidecar_path_for(path)
    };
    let bytes = std::fs::read(&sidecar_path).ok()?;
    let sidecar: QueueItemSidecar = serde_json::from_slice(&bytes).ok()?;
    let dt = chrono::DateTime::parse_from_rfc3339(&sidecar.enqueued_at).ok()?;
    let enqueued: std::time::SystemTime = dt.with_timezone(&chrono::Utc).into();
    Some(
        std::time::SystemTime::now()
            .duration_since(enqueued)
            .unwrap_or(Duration::ZERO),
    )
}

fn is_mismatched_queue_file(
    name: &std::ffi::OsStr,
    all_names: &HashSet<std::ffi::OsString>,
) -> bool {
    let name_str = name.to_string_lossy();
    if let Some(stem) = name_str.strip_suffix(SIDECAR_SUFFIX) {
        !all_names.contains(std::ffi::OsStr::new(stem))
    } else {
        let sidecar = format!("{name_str}{SIDECAR_SUFFIX}");
        !all_names.contains(std::ffi::OsStr::new(sidecar.as_str()))
    }
}

/// Reap `scratch/<sid>/` subdirs older than `max_age`. Returns
/// `(removed_count, removed_bytes)`.
fn cleanup_scratch_subdirs(scratch_dir: &Path, max_age: Duration) -> (u64, u64) {
    let entries = match std::fs::read_dir(scratch_dir) {
        Ok(e) => e,
        Err(_) => return (0, 0),
    };
    let mut cleaned = 0u64;
    let mut cleaned_bytes = 0u64;
    for entry in entries.flatten() {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let age = metadata
            .modified()
            .ok()
            .and_then(|m| m.elapsed().ok())
            .unwrap_or(Duration::MAX);
        if age <= max_age {
            continue;
        }
        let path = entry.path();
        if metadata.is_dir() {
            let size = dir_size(&path).unwrap_or(0);
            if std::fs::remove_dir_all(&path).is_ok() {
                cleaned += 1;
                cleaned_bytes += size;
            }
        } else if std::fs::remove_file(&path).is_ok() {
            cleaned += 1;
            cleaned_bytes += metadata.len();
        }
    }
    (cleaned, cleaned_bytes)
}

/// Recursively compute the total size of a directory tree.
fn dir_size(path: &Path) -> std::io::Result<u64> {
    let mut total = 0u64;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        if meta.is_dir() {
            total += dir_size(&entry.path())?;
        } else {
            total += meta.len();
        }
    }
    Ok(total)
}
