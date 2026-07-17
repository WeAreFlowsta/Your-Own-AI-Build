//! S3-compatible upload layer - removed from this build.
//!
//! This build ships without any cloud-upload capability. The public API
//! surface is preserved so callers compile, but every entry point returns
//! an error immediately and performs no network I/O.

use std::collections::HashSet;
use std::path::Path;

const UPLOAD_REMOVED: &str = "cloud upload removed from this build";

/// Static access-key credentials formerly used for presigning S3 URLs.
///
/// `Debug` is intentionally redacted - the struct holds plaintext secrets.
#[derive(Clone)]
pub struct S3StaticCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
}

impl std::fmt::Debug for S3StaticCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("S3StaticCredentials")
            .field("access_key_id", &"[redacted]")
            .field("secret_access_key", &"[redacted]")
            .finish()
    }
}

/// Cloud upload removed from this build: always returns an error, sends nothing.
pub async fn presign_put_url(
    _region: &str,
    _endpoint_url: Option<&str>,
    _creds: &S3StaticCredentials,
    _bucket: &str,
    _key: &str,
    _content_type: &str,
    _expires_in: std::time::Duration,
) -> anyhow::Result<String> {
    anyhow::bail!(UPLOAD_REMOVED)
}

/// Cloud upload removed from this build: always returns an error, sends nothing.
pub async fn presign_get_url(
    _region: &str,
    _endpoint_url: Option<&str>,
    _creds: &S3StaticCredentials,
    _bucket: &str,
    _key: &str,
    _expires_in: std::time::Duration,
) -> anyhow::Result<String> {
    anyhow::bail!(UPLOAD_REMOVED)
}

/// Cloud upload removed from this build: always returns an error, sends nothing.
#[allow(clippy::too_many_arguments)]
pub async fn upload_bytes(
    bucket: &str,
    object_path: &str,
    _content: &[u8],
    _content_type: &str,
    _region: &str,
    _credentials_content: Option<&str>,
    _credentials_file: Option<&str>,
    _endpoint_url: Option<&str>,
) -> anyhow::Result<String> {
    anyhow::bail!("{UPLOAD_REMOVED} (s3://{bucket}/{object_path})")
}

/// Cloud upload removed from this build: always returns an error, sends nothing.
#[allow(clippy::too_many_arguments)]
pub async fn upload_file(
    bucket: &str,
    object_path: &str,
    _file_path: &Path,
    _content_type: &str,
    _region: &str,
    _credentials_content: Option<&str>,
    _credentials_file: Option<&str>,
    _endpoint_url: Option<&str>,
) -> anyhow::Result<String> {
    anyhow::bail!("{UPLOAD_REMOVED} (s3://{bucket}/{object_path})")
}

/// Cloud upload removed from this build: always returns an error, sends nothing.
#[allow(clippy::too_many_arguments)]
pub async fn upload_stream<R: tokio::io::AsyncRead + Send + Sync + 'static>(
    bucket: &str,
    object_path: &str,
    _reader: R,
    _content_type: &str,
    _region: &str,
    _credentials_content: Option<&str>,
    _credentials_file: Option<&str>,
    _endpoint_url: Option<&str>,
) -> anyhow::Result<String> {
    anyhow::bail!("{UPLOAD_REMOVED} (s3://{bucket}/{object_path})")
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct S3ExistsResponse {
    pub bucket: String,
    pub path: String,
    pub size: i64,
}

/// Former S3-native storage client. Cloud upload removed from this build:
/// construction fails, and no method performs network I/O.
#[allow(dead_code)]
pub struct S3StorageClient {
    bucket: String,
}

#[allow(dead_code)]
impl S3StorageClient {
    pub fn bucket_name(&self) -> &str {
        &self.bucket
    }

    pub async fn new(
        _bucket: String,
        _region: &str,
        _credentials_content: Option<&str>,
        _credentials_file: Option<&str>,
        _endpoint_url: Option<&str>,
    ) -> anyhow::Result<Self> {
        anyhow::bail!(UPLOAD_REMOVED)
    }

    /// Cloud upload removed from this build: reports `ProbeFailed`, sends nothing.
    pub async fn batch_check_exists<S: AsRef<str>>(
        &self,
        _paths: &[S],
    ) -> crate::storage_client::ExistsResult<HashSet<String>> {
        crate::storage_client::ExistsResult::ProbeFailed
    }

    /// Cloud upload removed from this build: returns `None`, sends nothing.
    pub async fn batch_upload(
        &self,
        _files: Vec<(String, Vec<u8>, String)>,
    ) -> Option<Vec<prod_mc_cli_chat_proxy_types::BatchUploadResult>> {
        None
    }

    /// Cloud upload removed from this build: returns `None`, sends nothing.
    pub async fn check_exists(&self, _path: &str) -> Option<S3ExistsResponse> {
        None
    }
}
