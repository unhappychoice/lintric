use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

#[test]
fn html_reports_isolate_unicode_hash_collisions() {
    let fixture = HtmlFixture::new("unicode", &["一瘕.rs", "一籏.rs"]);
    fixture.render(&["一瘕.rs", "一籏.rs"]);
    assert_eq!(fixture.assert_pages(), ["rs-0.html", "rs-1.html"]);
}

#[test]
// Windows whole-path limits depend on host configuration; byte bounds are unit-tested everywhere.
#[cfg(unix)]
fn html_reports_bound_long_names_and_keep_truncated_slugs_unique() {
    let prefix = "a".repeat(240);
    let longer = "a".repeat(250);
    let names = [
        format!("{prefix}.rs"),
        format!("{longer}.rs"),
        format!("{longer}b.rs"),
    ];
    let paths: Vec<_> = names.iter().map(String::as_str).collect();
    let fixture = HtmlFixture::new("long", &paths);
    fixture.render(&paths);
    let links = fixture.assert_pages();
    assert!(links.iter().any(|name| name.len() == 255));
    links.iter().for_each(|name| {
        assert!(name.len() <= 255, "{} bytes: {name}", name.len());
        assert!(name.starts_with(&prefix));
    });
}

#[test]
fn html_reports_use_readable_stable_names_for_typical_paths() {
    let paths = ["src/a.rs", "src_a.rs", "index.rs"];
    let fixture = HtmlFixture::new("typical", &paths);
    fixture.render(&["."]);
    let first: HashSet<_> = fixture.assert_pages().into_iter().collect();
    assert_eq!(
        first,
        HashSet::from([
            "index_rs-0.html".into(),
            "src_a_rs-1.html".into(),
            "src_a_rs-2.html".into()
        ])
    );
    fixture.render(&["."]);
    assert_eq!(first, fixture.assert_pages().into_iter().collect());
}

#[test]
fn html_output_directory_failure_exits_nonzero() {
    let fixture = HtmlFixture::new("directory-failure", &["a.rs"]);
    fs::write(fixture.root.join(".lintric"), "blocked").unwrap();
    fixture.assert_failure(&["a.rs"], "Error creating output directory");
}

#[test]
fn html_index_write_failure_exits_nonzero() {
    let fixture = HtmlFixture::new("index-failure", &["a.rs"]);
    let output = fixture.root.join(".lintric/output/html");
    fs::create_dir_all(output.join("index.html")).unwrap();
    fixture.assert_failure(&["a.rs"], "Error writing index.html");
    assert!(output.join("a_rs-0.html").is_file());
}

#[test]
fn html_page_write_failure_exits_nonzero_and_keeps_other_reports() {
    let fixture = HtmlFixture::new("page-failure", &["a.rs", "b.rs"]);
    let output = fixture.root.join(".lintric/output/html");
    fs::create_dir_all(output.join("a_rs-0.html")).unwrap();
    fixture.assert_failure(&["a.rs", "b.rs"], "Error generating HTML for file a.rs");
    assert!(output.join("index.html").is_file());
    let page = fs::read_to_string(output.join("b_rs-1.html")).unwrap();
    assert!(page.contains("unique_source_1"));
}

#[test]
fn html_success_does_not_hide_analysis_failure() {
    let fixture = HtmlFixture::new("analysis-failure", &["a.rs"]);
    fixture.assert_failure(&["missing.rs", "a.rs"], "is neither a file nor a directory");
    fixture.assert_pages();
}

struct HtmlFixture {
    root: PathBuf,
    paths: Vec<String>,
}

impl HtmlFixture {
    fn new(label: &str, paths: &[&str]) -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/html-tests")
            .join(format!("{label}-{}", std::process::id()));
        paths.iter().enumerate().for_each(|(index, path)| {
            let file = root.join(path);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, format!("// unique_source_{index}\nfn main() {{}}\n")).unwrap();
        });
        Self {
            root,
            paths: paths.iter().map(|path| (*path).into()).collect(),
        }
    }

    fn render(&self, paths: &[&str]) {
        let output = self.run(paths);
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
    }

    fn run(&self, paths: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_lintric"))
            .current_dir(&self.root)
            .arg("--html")
            .args(paths)
            .output()
            .unwrap()
    }

    fn assert_failure(&self, paths: &[&str], message: &str) {
        let output = self.run(paths);
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(message),
            "{output:?}"
        );
    }

    fn assert_pages(&self) -> Vec<String> {
        let output = self.root.join(".lintric/output/html");
        let index = fs::read_to_string(output.join("index.html"))
            .unwrap()
            .replace('\\', "/");
        let links: Vec<_> = index
            .split("<a href=\"")
            .skip(1)
            .map(|link| link.split_once('"').unwrap().0.to_string())
            .collect();
        assert_eq!(links.len(), self.paths.len());
        assert_eq!(links.iter().collect::<HashSet<_>>().len(), links.len());
        self.paths.iter().enumerate().for_each(|(id, path)| {
            let link = links
                .iter()
                .find(|link| {
                    index.contains(&format!("href=\"{link}\" class=\"file-link\">{path}</a>"))
                })
                .unwrap();
            let page = fs::read_to_string(output.join(link))
                .unwrap()
                .replace('\\', "/");
            assert!(
                page.contains(&format!("Analysis for: {path}</h1>")),
                "{page}"
            );
            self.paths.iter().enumerate().for_each(|(other, _)| {
                assert_eq!(
                    page.contains(&format!("unique_source_{other}")),
                    id == other
                );
            });
        });
        links
    }
}

impl Drop for HtmlFixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
