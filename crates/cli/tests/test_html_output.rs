use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn html_reports_isolate_unicode_hash_collisions() {
    let fixture = HtmlFixture::new("unicode", &["一瘕.rs", "一籏.rs"]);
    fixture.render(&["一瘕.rs", "一籏.rs"]);
    assert_eq!(fixture.assert_pages(), ["rs-0.html", "rs-1.html"]);
}

#[test]
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
        let binary = option_env!("CARGO_BIN_EXE_lintric")
            .or(option_env!("CARGO_BIN_EXE_lintric-cli"))
            .expect("CLI binary must be built for integration tests");
        let output = Command::new(binary)
            .current_dir(&self.root)
            .arg("--html")
            .args(paths)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
    }

    fn assert_pages(&self) -> Vec<String> {
        let output = self.root.join(".lintric/output/html");
        let index = fs::read_to_string(output.join("index.html")).unwrap();
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
            let page = fs::read_to_string(output.join(link)).unwrap();
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
