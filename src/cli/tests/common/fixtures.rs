use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{TempDir, tempdir};
use walkdir::WalkDir;

const BASE_URL_PLACEHOLDER: &str = "__BASE_URL__";

pub struct FixtureWorkspace {
    root: TempDir,
}

impl FixtureWorkspace {
    pub fn new(base_url: &str) -> Result<Self> {
        let root = tempdir().context("create fixture workspace")?;
        copy_fixture_tree(&fixture_source_dir(), root.path(), base_url)?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        self.root.path()
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.root.path().join(relative)
    }

    pub fn arg(&self, relative: &str) -> String {
        self.path(relative).display().to_string()
    }

    #[allow(dead_code)]
    pub fn read_path(&self, path: &Path) -> Result<String> {
        fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
    }

    #[allow(dead_code)]
    pub fn generated_file(&self, prefix: &str, extension: &str) -> Result<PathBuf> {
        let matches = self.generated_files(prefix, extension)?;
        match matches.as_slice() {
            [single] => Ok(single.clone()),
            [] => Err(anyhow!(
                "no generated file found for prefix {prefix} and extension {extension}"
            )),
            _ => Err(anyhow!(
                "expected one generated file for prefix {prefix} and extension {extension}, found {}",
                matches.len()
            )),
        }
    }

    #[allow(dead_code)]
    pub fn generated_files(&self, prefix: &str, extension: &str) -> Result<Vec<PathBuf>> {
        let mut matches = Vec::new();
        for entry in fs::read_dir(self.root())? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };

            if file_name.starts_with(prefix) && file_name.ends_with(extension) {
                matches.push(path);
            }
        }

        matches.sort();
        Ok(matches)
    }
}

fn fixture_source_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn copy_fixture_tree(source: &Path, destination: &Path, base_url: &str) -> Result<()> {
    for entry in WalkDir::new(source) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(source)?;
        let target = destination.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).with_context(|| format!("create {}", target.display()))?;
            continue;
        }

        let contents = fs::read_to_string(entry.path())
            .with_context(|| format!("read {}", entry.path().display()))?;
        let localized = contents.replace(BASE_URL_PLACEHOLDER, base_url);
        fs::write(&target, localized).with_context(|| format!("write {}", target.display()))?;
    }

    Ok(())
}
