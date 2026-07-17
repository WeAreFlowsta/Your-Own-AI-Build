//! Storage client - cloud upload removed from this build.
//!
//! The former REST client uploaded files to cloud storage via a proxy.
//! This build ships without any cloud-upload capability: the public API
//! surface is preserved so callers compile, but every method returns an
//! error (or an empty/negative result) immediately and performs no
//! network I/O.

use anyhow::Result;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncRead;
use xai_grok_auth::AuthCredentialProvider;

const UPLOAD_REMOVED: &str = "cloud upload removed from this build";

/// Hook formerly invoked at every 401 response site. Retained so embedding
/// applications keep compiling; never called in this build (no requests are
/// made).
pub trait Auth401AttributionCallback: Send + Sync + std::fmt::Debug {
    fn record_401(&self, operation: &str, sent_bearer_prefix: Option<&str>);
}

/// Configuration for exponential backoff retry logic. Retained for API
/// compatibility; unused in this build (no requests are made).
#[derive(Debug, Clone)]
pub struct RetryConfig {
    #[allow(dead_code)]
    initial_delay: Duration,
    #[allow(dead_code)]
    max_delay: Duration,
    #[allow(dead_code)]
    max_retries: u32,
    #[allow(dead_code)]
    multiplier: f64,
    #[allow(dead_code)]
    jitter_factor: f64,
    #[allow(dead_code)]
    respect_retry_after: bool,
    #[allow(dead_code)]
    max_retry_after: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            max_retries: 5,
            multiplier: 2.0,
            jitter_factor: 0.5,
            respect_retry_after: true,
            max_retry_after: Duration::from_secs(60),
        }
    }
}

impl RetryConfig {
    /// Create a new RetryConfig with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a conservative config for high-reliability scenarios.
    pub fn conservative() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            max_retries: 8,
            multiplier: 2.0,
            jitter_factor: 0.5,
            respect_retry_after: true,
            max_retry_after: Duration::from_secs(120),
        }
    }

    /// Set the initial delay before the first retry.
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set the maximum delay between retries.
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set the maximum number of retry attempts.
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set the exponential backoff multiplier.
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Set the jitter factor (0.0 to 1.0).
    pub fn with_jitter_factor(mut self, factor: f64) -> Self {
        self.jitter_factor = factor.clamp(0.0, 1.0);
        self
    }
}

/// HTTP upload error with structured status code. Retained because callers
/// downcast to it.
#[derive(Debug)]
pub struct HttpUploadError {
    pub status_code: u16,
    pub message: String,
}

impl std::fmt::Display for HttpUploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for HttpUploadError {}

/// Outcome of a storage existence check. In this build every check reports
/// `ProbeFailed` (no probe is ever sent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExistsResult<T> {
    Found(T),
    NotFound,
    Unauthorized,
    ProbeFailed,
}

/// Response shape from the former upload endpoint. Retained for API
/// compatibility.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UploadResponse {
    pub bucket: String,
    pub path: String,
    pub size: i64,
    pub content_type: String,
    pub generation: i64,
}

/// Upload size limits shape. Retained for API compatibility.
#[derive(Debug, Clone, Deserialize)]
pub struct UploadLimits {
    pub max_file_bytes: u64,
    pub max_untracked_bytes: u64,
    pub enabled: bool,
}

/// Static credentials holder. Retained for API compatibility; only used to
/// carry values through constructors - never put on the wire in this build.
pub struct StaticGrokAuth {
    pub user_token: Option<String>,
    pub deployment_key: Option<String>,
}

impl StaticGrokAuth {
    pub fn new(user_token: Option<String>) -> Self {
        Self {
            user_token,
            deployment_key: None,
        }
    }

    /// Returns the bearer that would formerly have been put on the wire
    /// (deployment_key first, else user_token).
    pub fn wire_bearer(&self) -> Option<String> {
        self.deployment_key
            .clone()
            .or_else(|| self.user_token.clone())
    }
}

impl xai_grok_auth::HttpAuth for StaticGrokAuth {
    fn apply(&self, builder: reqwest::RequestBuilder, _base_url: &str) -> reqwest::RequestBuilder {
        // No storage requests are made in this build; header application is
        // retained only to satisfy the trait.
        builder
    }
}

/// Former client for uploading files to cloud storage via a proxy.
///
/// Cloud upload removed from this build: no method performs network I/O.
#[derive(Clone)]
pub struct StorageClient {
    #[allow(dead_code)]
    base_url: String,
    #[allow(dead_code)]
    retry_config: RetryConfig,
    #[allow(dead_code)]
    attribution: Option<Arc<dyn Auth401AttributionCallback>>,
    #[allow(dead_code)]
    credentials: Arc<dyn AuthCredentialProvider>,
    #[allow(dead_code)]
    client_version: Option<String>,
    #[allow(dead_code)]
    client_identifier: Option<String>,
    #[allow(dead_code)]
    client_mode: Option<String>,
}

impl StorageClient {
    /// Creates a new StorageClient with a static user-token credential.
    /// Cloud upload removed from this build: the client never sends anything.
    pub fn new(proxy_base_url: &str, user_token: &str) -> Self {
        let creds = StaticGrokAuth::new(Some(user_token.to_owned()));
        let bearer = creds.wire_bearer();
        let provider = Arc::new(xai_grok_auth::StaticAuthCredentialProvider::new(
            Box::new(creds),
            bearer,
        ));
        Self::with_provider(proxy_base_url, reqwest::Client::new(), provider)
    }

    /// Creates a new StorageClient. The HTTP client argument is accepted for
    /// signature compatibility and dropped - no requests are made.
    pub fn with_provider(
        proxy_base_url: &str,
        _http_client: reqwest::Client,
        credentials: Arc<dyn AuthCredentialProvider>,
    ) -> Self {
        Self {
            base_url: proxy_base_url.to_owned(),
            retry_config: RetryConfig::default(),
            attribution: None,
            credentials,
            client_version: None,
            client_identifier: None,
            client_mode: None,
        }
    }

    /// Always `false` - there is no breaker because there are no requests.
    pub fn storage_breaker_is_open(&self) -> bool {
        false
    }

    /// Retained for API compatibility; the callback is never invoked.
    pub fn with_attribution(mut self, callback: Arc<dyn Auth401AttributionCallback>) -> Self {
        self.attribution = Some(callback);
        self
    }

    /// Retained for API compatibility; the identity is never transmitted.
    pub fn with_client_identity(
        mut self,
        version: impl Into<String>,
        identifier: impl Into<String>,
    ) -> Self {
        self.client_version = Some(version.into());
        self.client_identifier = Some(identifier.into());
        self
    }

    /// Retained for API compatibility; the mode is never transmitted.
    pub fn with_client_mode(mut self, mode: impl Into<String>) -> Self {
        self.client_mode = Some(mode.into());
        self
    }

    /// Retained for API compatibility; there are no retries because there are
    /// no requests.
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Cloud upload removed from this build: always returns an error, sends nothing.
    pub async fn get_upload_limits(&self) -> Result<UploadLimits> {
        anyhow::bail!(UPLOAD_REMOVED)
    }

    /// Cloud upload removed from this build: reports `ProbeFailed`, sends nothing.
    pub async fn check_exists(&self, _path: &str) -> ExistsResult<UploadResponse> {
        ExistsResult::ProbeFailed
    }

    /// Cloud upload removed from this build: reports `ProbeFailed`, sends nothing.
    pub async fn batch_check_exists<S: AsRef<str>>(
        &self,
        _paths: &[S],
    ) -> ExistsResult<HashSet<String>> {
        ExistsResult::ProbeFailed
    }

    /// Cloud upload removed from this build: returns `None`, sends nothing.
    pub async fn batch_upload(
        &self,
        _files: Vec<(String, Vec<u8>, String)>,
    ) -> Option<Vec<prod_mc_cli_chat_proxy_types::BatchUploadResult>> {
        None
    }

    /// Cloud upload removed from this build: returns `None`, sends nothing.
    pub async fn batch_upload_json(
        &self,
        _files: Vec<(String, Vec<u8>, String)>,
    ) -> Option<Vec<prod_mc_cli_chat_proxy_types::BatchUploadResult>> {
        None
    }

    /// Cloud download removed from this build: always returns an error, fetches nothing.
    pub async fn download_blob(&self, _storage_path: &str, _dest: &Path) -> Result<()> {
        anyhow::bail!("cloud download removed from this build")
    }

    /// Cloud upload removed from this build: always returns an error, sends nothing.
    pub async fn upload(
        &self,
        path: &str,
        _content: &[u8],
        _content_type: &str,
    ) -> Result<UploadResponse> {
        Err(HttpUploadError {
            status_code: 501,
            message: format!("{UPLOAD_REMOVED} (path: {path})"),
        }
        .into())
    }

    /// Cloud upload removed from this build: always returns an error, sends nothing.
    pub async fn upload_file(
        &self,
        dest_path: &str,
        _file_path: &Path,
        _content_type: &str,
    ) -> Result<UploadResponse> {
        Err(HttpUploadError {
            status_code: 501,
            message: format!("{UPLOAD_REMOVED} (path: {dest_path})"),
        }
        .into())
    }

    /// Cloud upload removed from this build: always returns an error, sends nothing.
    pub async fn upload_stream<R>(
        &self,
        path: &str,
        _reader: R,
        _content_type: &str,
    ) -> Result<UploadResponse>
    where
        R: AsyncRead + Send + Sync + 'static,
    {
        anyhow::bail!("{UPLOAD_REMOVED} (path: {path})")
    }

    /// Cloud upload removed from this build: always returns an error, sends nothing.
    pub async fn upload_multipart(
        &self,
        path: &str,
        _file_path: &Path,
        _content_type: &str,
        _options: Option<MultipartUploadOptions>,
    ) -> Result<MultipartCompleteResponse> {
        anyhow::bail!("{UPLOAD_REMOVED} (path: {path})")
    }

    /// Cloud upload removed from this build: always returns an error, sends nothing.
    pub async fn get_signed_upload_url(
        &self,
        path: &str,
        _content_type: &str,
    ) -> Result<prod_mc_cli_chat_proxy_types::SignedUploadUrlResponse> {
        anyhow::bail!("{UPLOAD_REMOVED} (path: {path})")
    }

    /// Cloud upload removed from this build: always returns an error, sends nothing.
    pub async fn upload_via_signed_url(
        &self,
        _signed_url: &str,
        _data: &[u8],
        _content_type: &str,
    ) -> Result<()> {
        anyhow::bail!(UPLOAD_REMOVED)
    }

    /// Cloud upload removed from this build: always returns an error, sends nothing.
    pub async fn upload_bytes_signed(
        &self,
        path: &str,
        _data: &[u8],
        _content_type: &str,
    ) -> Result<prod_mc_cli_chat_proxy_types::SignedUploadUrlResponse> {
        anyhow::bail!("{UPLOAD_REMOVED} (path: {path})")
    }
}

/// Options for multipart uploads. Retained for API compatibility; unused in
/// this build.
#[derive(Debug, Clone)]
pub struct MultipartUploadOptions {
    pub part_size_bytes: Option<usize>,
    pub max_concurrent_uploads: usize,
}

impl Default for MultipartUploadOptions {
    fn default() -> Self {
        Self {
            part_size_bytes: None,
            max_concurrent_uploads: 4,
        }
    }
}

impl MultipartUploadOptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_part_size(mut self, size_bytes: usize) -> Self {
        self.part_size_bytes = Some(size_bytes);
        self
    }
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent_uploads = max;
        self
    }
}

/// Response from initializing a multipart upload. Retained for API
/// compatibility.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultipartInitResponse {
    pub upload_id: String,
    pub bucket: String,
    pub max_part_size_bytes: u64,
    #[serde(default)]
    pub part_urls: Vec<SignedPartUrl>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// Pre-signed URL information for a single part. Retained for API
/// compatibility.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedPartUrl {
    pub part_number: u32,
    pub url: String,
    pub path: String,
}

/// Response from uploading a single part. Retained for API compatibility.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultipartUploadPartResponse {
    pub part_number: u32,
    pub path: String,
    pub size: i64,
}

/// Information about an uploaded part. Retained for API compatibility.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadedPartInfo {
    pub part_number: u32,
    pub path: String,
}

/// Response from completing a multipart upload. Retained for API
/// compatibility.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultipartCompleteResponse {
    pub bucket: String,
    pub path: String,
    pub gcs_url: String,
    pub size: i64,
    pub parts_composed: usize,
    pub generation: i64,
}
