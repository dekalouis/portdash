use std::path::{Path, PathBuf};

const MARKERS: [&str; 10] = [
    ".git",
    "package.json",
    "Cargo.toml",
    "pyproject.toml",
    "requirements.txt",
    "go.mod",
    "composer.json",
    "Gemfile",
    "pom.xml",
    "build.gradle",
];

/// Finds the project that `start` belongs to, as its root folder and name.
///
/// Walks up from `start` and returns the nearest folder that holds a marker
/// file from `MARKERS`. The name is the root's folder name. The walk stops
/// before `home`, so the home folder is never a project. Returns `None` if no
/// marker is found, or if the root has no folder name (`/`).
pub fn find_project(start: &Path, home: Option<&Path>) -> Option<(PathBuf, String)> {
    let root = start
        .ancestors()
        .take_while(|dir| Some(*dir) != home)
        .find(|dir| MARKERS.iter().any(|marker| dir.join(marker).exists()))?;

    let name = root.file_name()?.to_string_lossy().into_owned();
    Some((root.to_path_buf(), name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Makes an empty folder with a name unique to the test, so parallel tests do not interfere.
    fn fresh_dir(test: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("portdash-{test}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn finds_a_marker_in_the_start_folder() {
        let root = fresh_dir("start-folder");
        fs::write(root.join("package.json"), "").unwrap();

        let found = find_project(&root, None);

        assert_eq!(
            found,
            Some((root.clone(), "portdash-start-folder".to_string()))
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn finds_a_marker_in_a_parent() {
        let root = fresh_dir("parent");
        let app = root.join("app");
        let src = app.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(app.join("Cargo.toml"), "").unwrap();

        let found = find_project(&src, None);

        assert_eq!(found, Some((app, "app".to_string())));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn prefers_the_nearest_of_two_markers() {
        let root = fresh_dir("nearest");
        let shop = root.join("shop");
        let frontend = shop.join("frontend");
        let src = frontend.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir(shop.join(".git")).unwrap();
        fs::write(frontend.join("package.json"), "").unwrap();

        let found = find_project(&src, None);

        assert_eq!(found, Some((frontend, "frontend".to_string())));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn returns_none_without_a_marker() {
        let root = fresh_dir("no-marker");
        let deep = root.join("a").join("b");
        fs::create_dir_all(&deep).unwrap();

        // Home is the test folder, so the walk never reaches the real /tmp or /.
        assert_eq!(find_project(&deep, Some(&root)), None);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn never_counts_home_as_a_project() {
        let home = fresh_dir("home");
        let src = home.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir(home.join(".git")).unwrap();

        assert_eq!(find_project(&src, Some(&home)), None);
        fs::remove_dir_all(&home).unwrap();
    }

    #[test]
    fn accepts_a_git_file_as_a_marker() {
        let root = fresh_dir("git-file");
        fs::write(root.join(".git"), "gitdir: elsewhere").unwrap();

        let found = find_project(&root, None);

        assert_eq!(found, Some((root.clone(), "portdash-git-file".to_string())));
        fs::remove_dir_all(&root).unwrap();
    }
}
