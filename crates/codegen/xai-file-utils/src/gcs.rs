//! Cloud upload layer - removed from this build.
//!
//! This build ships without any cloud-upload capability. The public API
//! surface is preserved so callers compile, but every upload entry point
//! returns an error immediately and performs no network I/O.

use std::path::Path;
use std::sync::Arc;

use crate::UploadMethod;
use xai_grok_auth::AuthCredentialProvider;

use crate::storage_client::Auth401AttributionCallback;

/// Threshold formerly used for switching to multipart upload (50 MB).
/// Retained because callers reference it for logging/branching.
pub const MULTIPART_UPLOAD_THRESHOLD: u64 = 50 * 1024 * 1024;

const UPLOAD_REMOVED: &str = "cloud upload removed from this build";

/// Implement `StorageConfig` for `TraceExportConfig` so existing callers
/// keep compiling.
impl StorageConfig for crate::TraceExportConfig {
    fn bucket_url(&self) -> &str {
        self.bucket_url.as_deref().unwrap_or("gs://placeholder")
    }

    fn upload_method(&self) -> &UploadMethod {
        &self.upload_method
    }
}

/// A trait for storage configuration that provides bucket URL and upload method.
/// Retained for API compatibility; no implementation performs network I/O in
/// this build.
pub trait StorageConfig {
    fn bucket_url(&self) -> &str;
    fn upload_method(&self) -> &UploadMethod;
    /// Retained for API compatibility; unused (no uploads happen).
    fn proxy_credentials(&self) -> Option<Arc<dyn AuthCredentialProvider>> {
        None
    }
    /// Retained for API compatibility; unused (no uploads happen).
    fn proxy_attribution(&self) -> Option<Arc<dyn Auth401AttributionCallback>> {
        None
    }
    /// Retained for API compatibility; unused (no uploads happen).
    fn proxy_http_client(&self) -> Option<reqwest::Client> {
        None
    }
}

/// Cloud upload removed from this build: always returns an error, sends nothing.
pub async fn upload_bytes<C: StorageConfig>(
    _config: &C,
    object_path: &str,
    _content: &[u8],
    _content_type: &str,
) -> anyhow::Result<String> {
    anyhow::bail!("{UPLOAD_REMOVED} (path: {object_path})")
}

/// Cloud upload removed from this build: always returns an error, sends nothing.
pub async fn upload_bytes_signed<C: StorageConfig>(
    _config: &C,
    object_path: &str,
    _content: &[u8],
    _content_type: &str,
) -> anyhow::Result<String> {
    anyhow::bail!("{UPLOAD_REMOVED} (path: {object_path})")
}

/// Cloud upload removed from this build: always returns an error, sends nothing.
pub async fn upload_file<C: StorageConfig>(
    _config: &C,
    object_path: &str,
    _file_path: &Path,
    _content_type: &str,
) -> anyhow::Result<String> {
    anyhow::bail!("{UPLOAD_REMOVED} (path: {object_path})")
}

/// Cloud upload removed from this build: always returns an error, sends nothing.
pub async fn upload_stream<C: StorageConfig, R>(
    _config: &C,
    object_path: &str,
    _reader: R,
    _content_type: &str,
) -> anyhow::Result<String>
where
    R: tokio::io::AsyncRead + Send + Sync + 'static,
{
    anyhow::bail!("{UPLOAD_REMOVED} (path: {object_path})")
}

/// Cloud upload removed from this build: always returns an error, sends nothing.
#[allow(clippy::too_many_arguments)]
pub async fn upload_bytes_via_signed_url(
    _proxy_base_url: &str,
    _user_token: &str,
    _deployment_key: Option<&str>,
    object_path: &str,
    _content: &[u8],
    _content_type: &str,
    _credentials: Option<Arc<dyn AuthCredentialProvider>>,
    _attribution: Option<Arc<dyn Auth401AttributionCallback>>,
    _http_client: Option<reqwest::Client>,
) -> anyhow::Result<String> {
    anyhow::bail!("{UPLOAD_REMOVED} (path: {object_path})")
}
