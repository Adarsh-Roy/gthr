use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Holds the cloned repo path and an optional TempDir for cleanup.
/// When TempDir is Some, dropping this struct removes the cloned repo.
pub struct RemoteRepo {
    pub root: PathBuf,
    _temp_dir: Option<TempDir>,
}

/// Verify that git is available on PATH.
fn check_git_available() -> Result<()> {
    let output = Command::new("git")
        .arg("--version")
        .output()
        .context("git is required for remote repository support. Install it from https://git-scm.com")?;
    if !output.status.success() {
        bail!("git is required for remote repository support. Install it from https://git-scm.com");
    }
    Ok(())
}

/// Extract repository name from a git URL.
/// Handles HTTPS (`https://host/user/repo.git`) and SSH (`git@host:user/repo.git`).
fn extract_repo_name(url: &str) -> Result<String> {
    let url = url.trim_end_matches('/');
    let segment = url.rsplit('/').next().unwrap_or(url);
    // Handle SSH-style URLs where the segment may contain ':'
    let segment = if segment.contains(':') {
        segment.rsplit(':').next().unwrap_or(segment)
    } else {
        segment
    };
    let name = segment.trim_end_matches(".git");
    if name.is_empty() {
        bail!("Could not extract repository name from URL: {url}");
    }
    // Ensure HTTPS-style URLs have a path beyond just the host
    if let Some(after_scheme) = url.find("://").map(|i| &url[i + 3..]) {
        if !after_scheme.contains('/') {
            bail!("Could not extract repository name from URL: {url}");
        }
    }
    Ok(name.to_string())
}

/// Shallow-clone a git repository into target_dir.
fn clone_repo(url: &str, target: &Path) -> Result<()> {
    eprintln!("Cloning {} into {}...", url, target.display());
    let output = Command::new("git")
        .args(["clone", "--depth", "1", url])
        .arg(target)
        .output()
        .context("Failed to run git clone")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git clone failed:\n{stderr}");
    }
    Ok(())
}

/// Clone a remote repo, returning a RemoteRepo that manages its lifetime.
/// - If `keep` is Some, clone into `keep_path/repo_name/` (persists after exit).
/// - If `keep` is None, clone into a temp directory (cleaned up on drop).
pub fn prepare_remote_repo(url: &str, keep: Option<&Path>) -> Result<RemoteRepo> {
    check_git_available()?;
    let repo_name = extract_repo_name(url)?;

    let (root, temp_dir) = if let Some(keep_path) = keep {
        std::fs::create_dir_all(keep_path)
            .with_context(|| format!("Failed to create directory: {}", keep_path.display()))?;
        let target = keep_path.join(&repo_name);
        clone_repo(url, &target)?;
        (target, None)
    } else {
        let td = TempDir::new().context("Failed to create temporary directory")?;
        let target = td.path().join(&repo_name);
        clone_repo(url, &target)?;
        (target, Some(td))
    };

    Ok(RemoteRepo {
        root,
        _temp_dir: temp_dir,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_repo_name_https() {
        assert_eq!(
            extract_repo_name("https://github.com/user/repo").unwrap(),
            "repo"
        );
    }

    #[test]
    fn test_extract_repo_name_https_with_git_suffix() {
        assert_eq!(
            extract_repo_name("https://github.com/user/repo.git").unwrap(),
            "repo"
        );
    }

    #[test]
    fn test_extract_repo_name_ssh() {
        assert_eq!(
            extract_repo_name("git@github.com:user/repo.git").unwrap(),
            "repo"
        );
    }

    #[test]
    fn test_extract_repo_name_trailing_slash() {
        assert_eq!(
            extract_repo_name("https://github.com/user/repo/").unwrap(),
            "repo"
        );
    }

    #[test]
    fn test_extract_repo_name_nested_path() {
        assert_eq!(
            extract_repo_name("https://gitlab.com/group/sub/repo").unwrap(),
            "repo"
        );
    }

    #[test]
    fn test_extract_repo_name_empty_fails() {
        assert!(extract_repo_name("").is_err());
        assert!(extract_repo_name("https://github.com/").is_err());
    }
}
