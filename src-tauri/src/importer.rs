use anyhow::{Context, Result};
use scraper::{Html, Selector};
use std::path::{Path, PathBuf};

pub fn import_html(html_path: &Path, dest_dir: &Path) -> Result<ImportStats> {
    let content = std::fs::read_to_string(html_path)
        .with_context(|| format!("Failed to read {}", html_path.display()))?;

    std::fs::create_dir_all(dest_dir)
        .with_context(|| format!("Failed to create {}", dest_dir.display()))?;

    let document = Html::parse_document(&content);
    let mut stats = ImportStats::default();
    let root = dest_dir.to_path_buf();

    parse_dl(&document, &root, &mut stats)?;
    Ok(stats)
}

fn parse_dl(document: &Html, current_dir: &Path, stats: &mut ImportStats) -> Result<()> {
    let dl_sel = Selector::parse("dl").unwrap();

    // Find the root <DL> — the one that is a direct child of <BODY> or the first one
    let Some(root_dl) = document.select(&dl_sel).next() else {
        return Ok(());
    };

    walk_dl(root_dl, current_dir, stats)
}

fn walk_dl(dl: scraper::ElementRef, dir: &Path, stats: &mut ImportStats) -> Result<()> {
    let dt_sel = Selector::parse("dt").unwrap();
    let h3_sel = Selector::parse("h3").unwrap();
    let a_sel = Selector::parse("a").unwrap();
    let dl_sel = Selector::parse("dl").unwrap();

    for dt in dl.select(&dt_sel) {
        // Skip DTs that are nested deeper (inside child DLs)
        if is_direct_child_dt(dt, dl) {
            if let Some(h3) = dt.select(&h3_sel).next() {
                // It's a folder
                let folder_name = sanitize_name(&h3.text().collect::<String>());
                if folder_name.is_empty() {
                    continue;
                }
                let folder_path = unique_path(dir, &folder_name, true);
                std::fs::create_dir_all(&folder_path)?;
                stats.folders += 1;

                // Recurse into the sibling <DL>
                if let Some(child_dl) = dt.select(&dl_sel).next() {
                    walk_dl(child_dl, &folder_path, stats)?;
                }
            } else if let Some(a) = dt.select(&a_sel).next() {
                // It's a bookmark
                let title = a.text().collect::<String>();
                let title = sanitize_name(title.trim());
                let url = a.value().attr("href").unwrap_or("").to_string();
                if url.is_empty() {
                    continue;
                }
                let filename = if title.is_empty() {
                    sanitize_name(&url)
                } else {
                    title
                };
                let file_path = unique_path(dir, &filename, false);
                let content = format!("[InternetShortcut]\nURL={}\n", url);
                std::fs::write(&file_path, content)?;
                stats.bookmarks += 1;
            }
        }
    }
    Ok(())
}

fn is_direct_child_dt(dt: scraper::ElementRef, dl: scraper::ElementRef) -> bool {
    dt.parent()
        .and_then(|p| scraper::ElementRef::wrap(p))
        .map(|parent| parent.id() == dl.id())
        .unwrap_or(false)
}

fn sanitize_name(name: &str) -> String {
    let forbidden = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    let mut result: String = name
        .chars()
        .map(|c| if forbidden.contains(&c) { '_' } else { c })
        .collect();
    result = result.trim().to_string();
    if result.len() > 200 {
        result.truncate(200);
    }
    result
}

fn unique_path(dir: &Path, name: &str, is_dir: bool) -> PathBuf {
    let ext = if is_dir { "" } else { ".url" };
    let base = dir.join(format!("{}{}", name, ext));
    if !base.exists() {
        return base;
    }
    let mut i = 2;
    loop {
        let candidate = dir.join(format!("{} ({}){}", name, i, ext));
        if !candidate.exists() {
            return candidate;
        }
        i += 1;
    }
}

#[derive(Debug, Default, serde::Serialize)]
pub struct ImportStats {
    pub folders: usize,
    pub bookmarks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const SIMPLE_HTML: &str = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<META HTTP-EQUIV="Content-Type" CONTENT="text/html; charset=UTF-8">
<TITLE>Bookmarks</TITLE><H1>Bookmarks</H1>
<DL><p>
    <DT><H3>Engineering</H3>
    <DL><p>
        <DT><A HREF="https://rust-lang.org">The Rust Language</A>
        <DT><A HREF="https://github.com">GitHub</A>
    </DL><p>
    <DT><A HREF="https://example.com">Example Site</A>
</DL><p>"#;

    const NESTED_HTML: &str = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<DL><p>
    <DT><H3>A</H3>
    <DL><p>
        <DT><H3>B</H3>
        <DL><p>
            <DT><H3>C</H3>
            <DL><p>
                <DT><A HREF="https://deep.example.com">Deep</A>
            </DL><p>
        </DL><p>
    </DL><p>
</DL><p>"#;

    const DIRTY_HTML: &str = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<DL><p>
    <DT><A HREF="https://a.com">title/with:slash*and?more"chars<>|</A>
    <DT><A HREF="https://b.com">  leading trailing spaces  </A>
    <DT><A HREF="">no url bookmark</A>
    <DT><H3>  </H3>
</DL><p>"#;

    fn write_html(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let path = dir.path().join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_flat_and_nested_import() {
        let tmp = TempDir::new().unwrap();
        let html = write_html(&tmp, "bm.html", SIMPLE_HTML);
        let dest = tmp.path().join("out");

        let stats = import_html(&html, &dest).unwrap();

        assert_eq!(stats.folders, 1);
        assert_eq!(stats.bookmarks, 3);
        assert!(dest.join("Engineering").is_dir());
        assert!(dest.join("Engineering/The Rust Language.url").exists());
        assert!(dest.join("Engineering/GitHub.url").exists());
        assert!(dest.join("Example Site.url").exists());
    }

    #[test]
    fn test_url_file_content() {
        let tmp = TempDir::new().unwrap();
        let html = write_html(&tmp, "bm.html", SIMPLE_HTML);
        let dest = tmp.path().join("out");
        import_html(&html, &dest).unwrap();

        let content = std::fs::read_to_string(dest.join("Example Site.url")).unwrap();
        assert!(content.contains("URL=https://example.com"));
    }

    #[test]
    fn test_three_level_nesting() {
        let tmp = TempDir::new().unwrap();
        let html = write_html(&tmp, "bm.html", NESTED_HTML);
        let dest = tmp.path().join("out");

        let stats = import_html(&html, &dest).unwrap();

        assert_eq!(stats.folders, 3);
        assert_eq!(stats.bookmarks, 1);
        assert!(dest.join("A/B/C/Deep.url").exists());
    }

    #[test]
    fn test_dirty_titles_sanitized() {
        let tmp = TempDir::new().unwrap();
        let html = write_html(&tmp, "bm.html", DIRTY_HTML);
        let dest = tmp.path().join("out");

        // Should not panic on dirty input
        let stats = import_html(&html, &dest).unwrap();

        // 2 valid bookmarks (empty URL and blank folder name skipped)
        assert_eq!(stats.bookmarks, 2);
        assert_eq!(stats.folders, 0);

        // Forbidden chars replaced with _
        let entries: Vec<_> = std::fs::read_dir(&dest)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        for name in &entries {
            for ch in ['/', '\\', ':', '*', '?', '"', '<', '>', '|'] {
                assert!(!name.contains(ch), "forbidden char {ch:?} in {name}");
            }
        }
    }

    #[test]
    fn test_empty_html_no_panic() {
        let tmp = TempDir::new().unwrap();
        let html = write_html(&tmp, "bm.html", "<html><body></body></html>");
        let dest = tmp.path().join("out");
        let stats = import_html(&html, &dest).unwrap();
        assert_eq!(stats.bookmarks, 0);
        assert_eq!(stats.folders, 0);
    }

    #[test]
    fn test_sanitize_name_forbidden_chars() {
        let forbidden = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        for ch in forbidden {
            let input = format!("pre{ch}post");
            let result = sanitize_name(&input);
            assert_eq!(result, "pre_post", "char {ch:?} should be replaced");
        }
    }

    #[test]
    fn test_sanitize_name_truncates_at_200() {
        let long = "a".repeat(300);
        assert_eq!(sanitize_name(&long).len(), 200);
    }

    #[test]
    fn test_sanitize_name_trims_whitespace() {
        assert_eq!(sanitize_name("  hello  "), "hello");
        assert_eq!(sanitize_name("\ttab\t"), "tab");
    }

    #[test]
    fn test_unique_path_avoids_collision() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("dup.url"), "").unwrap();

        let p2 = unique_path(tmp.path(), "dup", false);
        assert_eq!(p2.file_name().unwrap().to_str().unwrap(), "dup (2).url");

        std::fs::write(&p2, "").unwrap();
        let p3 = unique_path(tmp.path(), "dup", false);
        assert_eq!(p3.file_name().unwrap().to_str().unwrap(), "dup (3).url");
    }

    #[test]
    fn test_nonexistent_html_returns_error() {
        let tmp = TempDir::new().unwrap();
        let result = import_html(
            &tmp.path().join("does_not_exist.html"),
            &tmp.path().join("out"),
        );
        assert!(result.is_err());
    }
}
