use anyhow::Result;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub title: String,
    pub url: String,
    pub status: CheckStatus,
    pub final_url: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Ok,
    Redirect,
    Broken,
    Timeout,
    Error,
}

pub async fn check_all(
    dir: &Path,
    concurrency: usize,
    timeout_secs: u64,
) -> Result<Vec<CheckResult>> {
    let items = collect_urls(dir)?;
    if items.is_empty() {
        return Ok(vec![]);
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent("Mozilla/5.0 BookmarkShelf/0.1")
        .danger_accept_invalid_certs(false)
        .build()?;

    let results = stream::iter(items)
        .map(|(title, url)| {
            let client = client.clone();
            async move { check_one(&client, title, url).await }
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await;

    Ok(results)
}

fn canonical(url: &str) -> &str {
    url.trim_end_matches('/')
}

async fn check_one(client: &Client, title: String, url: String) -> CheckResult {
    let original_url = url.clone();
    match client.get(&url).send().await {
        Ok(resp) => {
            let status_code = resp.status().as_u16();
            let final_url = resp.url().to_string();
            // Treat trailing-slash normalisation as non-redirect
            let redirected = canonical(&final_url) != canonical(&original_url);

            if resp.status().is_success() || resp.status().is_redirection() {
                CheckResult {
                    title,
                    url: original_url.clone(),
                    status: if redirected {
                        CheckStatus::Redirect
                    } else {
                        CheckStatus::Ok
                    },
                    final_url: if redirected { Some(final_url) } else { None },
                    error: None,
                }
            } else {
                CheckResult {
                    title,
                    url: original_url,
                    status: CheckStatus::Broken,
                    final_url: None,
                    error: Some(format!("HTTP {}", status_code)),
                }
            }
        }
        Err(e) if e.is_timeout() => CheckResult {
            title,
            url: original_url,
            status: CheckStatus::Timeout,
            final_url: None,
            error: Some("Timeout".to_string()),
        },
        Err(e) => CheckResult {
            title,
            url: original_url,
            status: CheckStatus::Error,
            final_url: None,
            error: Some(e.to_string()),
        },
    }
}

fn collect_urls(dir: &Path) -> Result<Vec<(String, String)>> {
    let mut items = Vec::new();
    collect_recursive(dir, &mut items)?;
    Ok(items)
}

pub(crate) fn collect_recursive(dir: &Path, items: &mut Vec<(String, String)>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_recursive(&path, items)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("url") {
            let title = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let content = std::fs::read_to_string(&path)?;
            for line in content.lines() {
                if let Some(url) = line.trim().strip_prefix("URL=") {
                    items.push((title.clone(), url.to_string()));
                    break;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_url_files(tmp: &TempDir, entries: &[(&str, &str)]) {
        for (title, url) in entries {
            let content = format!("[InternetShortcut]\nURL={}\n", url);
            std::fs::write(tmp.path().join(format!("{}.url", title)), content).unwrap();
        }
    }

    // ── collect_urls ──────────────────────────────────────────────

    #[test]
    fn test_collect_flat() {
        let tmp = TempDir::new().unwrap();
        make_url_files(&tmp, &[("foo", "https://foo.com"), ("bar", "https://bar.com")]);

        let mut items = Vec::new();
        collect_recursive(tmp.path(), &mut items).unwrap();
        assert_eq!(items.len(), 2);
        let urls: Vec<_> = items.iter().map(|(_, u)| u.as_str()).collect();
        assert!(urls.contains(&"https://foo.com"));
        assert!(urls.contains(&"https://bar.com"));
    }

    #[test]
    fn test_collect_recursive_subdirs() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("deep.url"), "[InternetShortcut]\nURL=https://deep.com\n").unwrap();
        std::fs::write(tmp.path().join("root.url"), "[InternetShortcut]\nURL=https://root.com\n").unwrap();

        let mut items = Vec::new();
        collect_recursive(tmp.path(), &mut items).unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_collect_skips_files_without_url_key() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("broken.url"), "not a url file\n").unwrap();

        let mut items = Vec::new();
        collect_recursive(tmp.path(), &mut items).unwrap();
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_collect_ignores_non_url_files() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("readme.txt"), "hello").unwrap();
        std::fs::write(tmp.path().join("image.png"), "data").unwrap();

        let mut items = Vec::new();
        collect_recursive(tmp.path(), &mut items).unwrap();
        assert_eq!(items.len(), 0);
    }

    // ── check_all with mock HTTP server ──────────────────────────

    #[tokio::test]
    async fn test_http_200_classified_ok() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let tmp = TempDir::new().unwrap();
        make_url_files(&tmp, &[("site", &server.uri())]);

        let results = check_all(tmp.path(), 1, 5).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, CheckStatus::Ok);
        assert!(results[0].error.is_none());
    }

    #[tokio::test]
    async fn test_http_404_classified_broken() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let tmp = TempDir::new().unwrap();
        make_url_files(&tmp, &[("gone", &server.uri())]);

        let results = check_all(tmp.path(), 1, 5).await.unwrap();
        assert_eq!(results[0].status, CheckStatus::Broken);
        assert_eq!(results[0].error.as_deref(), Some("HTTP 404"));
    }

    #[tokio::test]
    async fn test_http_500_classified_broken() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let tmp = TempDir::new().unwrap();
        make_url_files(&tmp, &[("err", &server.uri())]);

        let results = check_all(tmp.path(), 1, 5).await.unwrap();
        assert_eq!(results[0].status, CheckStatus::Broken);
    }

    #[tokio::test]
    async fn test_unreachable_url_classified_error() {
        let tmp = TempDir::new().unwrap();
        // Port 1 is unlikely to be open
        make_url_files(&tmp, &[("dead", "http://127.0.0.1:1/nothing")]);

        let results = check_all(tmp.path(), 1, 2).await.unwrap();
        assert!(
            results[0].status == CheckStatus::Error || results[0].status == CheckStatus::Timeout,
            "expected Error or Timeout, got {:?}",
            results[0].status
        );
    }

    #[tokio::test]
    async fn test_empty_dir_returns_empty_results() {
        let tmp = TempDir::new().unwrap();
        let results = check_all(tmp.path(), 5, 5).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_urls_concurrently() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let tmp = TempDir::new().unwrap();
        for i in 0..10 {
            let content = format!("[InternetShortcut]\nURL={}\n", server.uri());
            std::fs::write(tmp.path().join(format!("bm{i}.url")), content).unwrap();
        }

        let results = check_all(tmp.path(), 5, 5).await.unwrap();
        assert_eq!(results.len(), 10);
        assert!(results.iter().all(|r| r.status == CheckStatus::Ok));
    }
}
