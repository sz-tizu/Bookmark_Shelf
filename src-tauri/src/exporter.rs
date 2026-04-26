use anyhow::{Context, Result};
use chrono::Local;
use std::fs;
use std::path::Path;

pub fn export_to_html(src_dir: &Path, output_path: &Path) -> Result<ExportStats> {
    let mut stats = ExportStats::default();
    let mut html = String::from(
        "<!DOCTYPE NETSCAPE-Bookmark-file-1>\n\
         <!-- This is an automatically generated file.\n\
              It will be read and overwritten.\n\
              DO NOT EDIT! -->\n\
         <META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n\
         <meta http-equiv=\"Content-Security-Policy\"\n\
               content=\"default-src 'self'; script-src 'none'; img-src * data:\">\n\
         <TITLE>Bookmarks</TITLE>\n\
         <H1>Bookmarks</H1>\n\
         <DL><p>\n",
    );

    walk_dir(src_dir, &mut html, 1, &mut stats)?;

    html.push_str("</DL><p>\n");

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, &html)
        .with_context(|| format!("Failed to write {}", output_path.display()))?;

    Ok(stats)
}

fn walk_dir(dir: &Path, html: &mut String, depth: usize, stats: &mut ExportStats) -> Result<()> {
    let indent = "    ".repeat(depth);
    let mut entries: Vec<_> = fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            let add_date = file_timestamp(&path);
            html.push_str(&format!(
                "{}<DT><H3 ADD_DATE=\"{}\">{}</H3>\n{}<DL><p>\n",
                indent,
                add_date,
                html_escape(&name),
                indent
            ));
            walk_dir(&path, html, depth + 1, stats)?;
            html.push_str(&format!("{}</DL><p>\n", indent));
            stats.folders += 1;
        } else if name.ends_with(".url") {
            let title = name.trim_end_matches(".url");
            let url = read_url_file(&path)?;
            if url.is_empty() {
                continue;
            }
            let add_date = file_timestamp(&path);
            html.push_str(&format!(
                "{}<DT><A HREF=\"{}\" ADD_DATE=\"{}\">{}</A>\n",
                indent,
                html_escape(&url),
                add_date,
                html_escape(title)
            ));
            stats.bookmarks += 1;
        }
    }
    Ok(())
}

fn read_url_file(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    for line in content.lines() {
        let line = line.trim();
        if let Some(url) = line.strip_prefix("URL=") {
            return Ok(url.to_string());
        }
    }
    Ok(String::new())
}

fn file_timestamp(path: &Path) -> i64 {
    path.metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or_else(|| Local::now().timestamp())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[derive(Debug, Default, serde::Serialize)]
pub struct ExportStats {
    pub folders: usize,
    pub bookmarks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_dir_tree(tmp: &TempDir) {
        // root/
        //   Engineering/
        //     Rust.url
        //     GitHub.url
        //   Example.url
        let eng = tmp.path().join("Engineering");
        std::fs::create_dir(&eng).unwrap();
        std::fs::write(eng.join("Rust.url"), "[InternetShortcut]\nURL=https://rust-lang.org\n").unwrap();
        std::fs::write(eng.join("GitHub.url"), "[InternetShortcut]\nURL=https://github.com\n").unwrap();
        std::fs::write(tmp.path().join("Example.url"), "[InternetShortcut]\nURL=https://example.com\n").unwrap();
    }

    #[test]
    fn test_export_doctype_and_structure() {
        let src = TempDir::new().unwrap();
        let out_dir = TempDir::new().unwrap();
        make_dir_tree(&src);

        let out = out_dir.path().join("out.html");
        export_to_html(src.path(), &out).unwrap();

        let html = std::fs::read_to_string(&out).unwrap();
        assert!(html.starts_with("<!DOCTYPE NETSCAPE-Bookmark-file-1>"));
        assert!(html.contains("<DL>"));
        assert!(html.contains("</DL>"));
    }

    #[test]
    fn test_export_stats() {
        let src = TempDir::new().unwrap();
        let out_dir = TempDir::new().unwrap();
        make_dir_tree(&src);

        let stats = export_to_html(src.path(), &out_dir.path().join("out.html")).unwrap();
        assert_eq!(stats.folders, 1);
        assert_eq!(stats.bookmarks, 3);
    }

    #[test]
    fn test_export_contains_all_urls() {
        let src = TempDir::new().unwrap();
        let out_dir = TempDir::new().unwrap();
        make_dir_tree(&src);

        let out = out_dir.path().join("out.html");
        export_to_html(src.path(), &out).unwrap();
        let html = std::fs::read_to_string(&out).unwrap();

        assert!(html.contains("https://rust-lang.org"), "rust url missing");
        assert!(html.contains("https://github.com"), "github url missing");
        assert!(html.contains("https://example.com"), "example url missing");
        assert!(html.contains("Engineering"), "folder name missing");
        assert!(html.contains("Rust"), "bookmark title missing");
    }

    #[test]
    fn test_export_empty_dir_no_crash() {
        let src = TempDir::new().unwrap();
        let out_dir = TempDir::new().unwrap();
        let stats = export_to_html(src.path(), &out_dir.path().join("out.html")).unwrap();
        assert_eq!(stats.bookmarks, 0);
        assert_eq!(stats.folders, 0);
    }

    #[test]
    fn test_export_url_file_without_url_key_skipped() {
        let src = TempDir::new().unwrap();
        std::fs::write(src.path().join("bad.url"), "not a url file content\n").unwrap();
        let out_dir = TempDir::new().unwrap();
        let stats = export_to_html(src.path(), &out_dir.path().join("out.html")).unwrap();
        assert_eq!(stats.bookmarks, 0);
    }

    #[test]
    fn test_html_escape_ampersand() {
        assert_eq!(html_escape("a&b"), "a&amp;b");
    }

    #[test]
    fn test_html_escape_brackets() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn test_html_escape_quote() {
        assert_eq!(html_escape("say \"hi\""), "say &quot;hi&quot;");
    }

    #[test]
    fn test_html_escape_no_change_plain() {
        assert_eq!(html_escape("hello world"), "hello world");
    }

    #[test]
    fn test_nonexistent_src_returns_error() {
        let out_dir = TempDir::new().unwrap();
        let result = export_to_html(
            &std::path::PathBuf::from("/does/not/exist"),
            &out_dir.path().join("out.html"),
        );
        assert!(result.is_err());
    }
}
