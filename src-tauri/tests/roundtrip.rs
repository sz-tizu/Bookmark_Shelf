/// Integration tests: import HTML → directory → export HTML
/// Verifies that the full data pipeline preserves bookmark integrity.
use bookmark_shelf_lib::{exporter, importer};
use tempfile::TempDir;

// ── helpers ──────────────────────────────────────────────────────────────

fn write_html(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path
}

fn export_html(src: &TempDir) -> String {
    let out_dir = TempDir::new().unwrap();
    let out = out_dir.path().join("out.html");
    exporter::export_to_html(src.path(), &out).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    // keep out_dir alive until string is read
    drop(out_dir);
    content
}

fn import_then_export(html: &str) -> (TempDir, String) {
    let tmp_in = TempDir::new().unwrap();
    let tmp_dir = TempDir::new().unwrap();
    let html_path = write_html(&tmp_in, "input.html", html);
    importer::import_html(&html_path, tmp_dir.path()).unwrap();
    let exported = export_html(&tmp_dir);
    (tmp_dir, exported)
}

// ── tests ────────────────────────────────────────────────────────────────

#[test]
fn roundtrip_all_urls_preserved() {
    let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<DL><p>
    <DT><A HREF="https://rust-lang.org">Rust</A>
    <DT><A HREF="https://github.com">GitHub</A>
    <DT><A HREF="https://example.com">Example</A>
</DL><p>"#;

    let (_dir, exported) = import_then_export(html);

    for url in &["https://rust-lang.org", "https://github.com", "https://example.com"] {
        assert!(exported.contains(url), "missing URL: {url}");
    }
}

#[test]
fn roundtrip_nested_folders_preserved() {
    let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<DL><p>
    <DT><H3>Level1</H3>
    <DL><p>
        <DT><H3>Level2</H3>
        <DL><p>
            <DT><A HREF="https://deep.example.com">Deep</A>
        </DL><p>
    </DL><p>
</DL><p>"#;

    let (dir, exported) = import_then_export(html);

    // Directory structure is correct
    assert!(dir.path().join("Level1/Level2/Deep.url").exists());

    // Exported HTML contains all the right pieces
    assert!(exported.contains("Level1"), "folder Level1 missing from export");
    assert!(exported.contains("Level2"), "folder Level2 missing from export");
    assert!(exported.contains("https://deep.example.com"), "URL missing from export");
}

#[test]
fn roundtrip_dirty_titles_survive() {
    // Titles with forbidden filesystem chars should be sanitized and still round-trip
    let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<DL><p>
    <DT><A HREF="https://a.com">C++ / Rust: The Book</A>
    <DT><A HREF="https://b.com">Q&amp;A Site</A>
</DL><p>"#;

    let (_dir, exported) = import_then_export(html);

    assert!(exported.contains("https://a.com"), "a.com URL missing");
    assert!(exported.contains("https://b.com"), "b.com URL missing");
}

#[test]
fn roundtrip_no_bookmark_loss() {
    let urls: Vec<String> = (1..=20)
        .map(|i| format!("https://site{i}.example.com"))
        .collect();

    let items: String = urls
        .iter()
        .enumerate()
        .map(|(i, u)| format!("    <DT><A HREF=\"{u}\">Site {i}</A>\n"))
        .collect();

    let html = format!(
        "<!DOCTYPE NETSCAPE-Bookmark-file-1>\n<DL><p>\n{}</DL><p>",
        items
    );

    let (_dir, exported) = import_then_export(&html);

    for url in &urls {
        assert!(exported.contains(url.as_str()), "lost URL: {url}");
    }
}

#[test]
fn roundtrip_empty_export_is_valid_html() {
    let tmp = TempDir::new().unwrap();
    let out_dir = TempDir::new().unwrap();
    let out = out_dir.path().join("out.html");

    exporter::export_to_html(tmp.path(), &out).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();

    assert!(content.contains("<!DOCTYPE NETSCAPE-Bookmark-file-1>"));
    assert!(content.contains("<DL>"));
    assert!(content.contains("</DL>"));
}

#[test]
fn roundtrip_mixed_folders_and_bookmarks() {
    let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<DL><p>
    <DT><H3>Work</H3>
    <DL><p>
        <DT><A HREF="https://work1.com">Work1</A>
        <DT><A HREF="https://work2.com">Work2</A>
    </DL><p>
    <DT><H3>Personal</H3>
    <DL><p>
        <DT><A HREF="https://personal1.com">Personal1</A>
    </DL><p>
    <DT><A HREF="https://root.com">Root Bookmark</A>
</DL><p>"#;

    let (dir, exported) = import_then_export(html);

    // Structure on disk
    assert!(dir.path().join("Work").is_dir());
    assert!(dir.path().join("Personal").is_dir());
    assert!(dir.path().join("Work/Work1.url").exists());
    assert!(dir.path().join("Personal/Personal1.url").exists());
    assert!(dir.path().join("Root Bookmark.url").exists());

    // All URLs survive export
    for url in &[
        "https://work1.com",
        "https://work2.com",
        "https://personal1.com",
        "https://root.com",
    ] {
        assert!(exported.contains(url), "missing: {url}");
    }
}
