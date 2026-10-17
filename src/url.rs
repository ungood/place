//! Clone URLs: comparing them and deriving a checkout path from them.

/// Normalize a URL for comparison. The scheme, the user, a trailing `.git`,
/// a trailing slash and case are ignored, and scp-style `host:path` is read
/// as `host/path`.
pub fn normalize(url: &str) -> String {
    let url = url.trim();
    let rest = match url.split_once("://") {
        Some((_, rest)) => rest.to_string(),
        None => match scp_parts(url) {
            Some((host, path)) => format!("{host}/{path}"),
            None => url.to_string(),
        },
    };
    let rest = match rest.split_once('/') {
        Some((authority, path)) => {
            let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
            format!("{host}/{path}")
        }
        None => rest,
    };
    let rest = rest.trim_end_matches('/');
    let rest = rest.strip_suffix(".git").unwrap_or(rest);
    rest.trim_end_matches('/').to_lowercase()
}

/// Whether two URLs name the same repository.
pub fn same(a: &str, b: &str) -> bool {
    normalize(a) == normalize(b)
}

/// Split an scp-style URL such as `git@github.com:owner/repo.git` into its
/// host (with any user) and path.
fn scp_parts(url: &str) -> Option<(&str, &str)> {
    let (host, path) = url.split_once(':')?;
    if host.is_empty() || host.contains('/') || path.starts_with("//") {
        return None;
    }
    Some((host, path))
}

/// Whether `target` looks like a URL rather than a local path.
pub fn is_url(target: &str) -> bool {
    target.contains("://") || scp_parts(target).is_some()
}

/// The directory `git clone` would create for a URL: its last path segment
/// without `.git`.
pub fn default_path(url: &str) -> Option<String> {
    let path = match url.split_once("://") {
        Some((_, rest)) => rest.split_once('/').map_or("", |(_, p)| p),
        None => scp_parts(url).map_or(url, |(_, p)| p),
    };
    let name = path.trim_end_matches('/').rsplit('/').next()?;
    let name = name.strip_suffix(".git").unwrap_or(name);
    (!name.is_empty()).then(|| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forms_of_the_same_url_match() {
        let canonical = "https://github.com/ungood/place.git";
        for other in [
            "git@github.com:ungood/place.git",
            "ssh://git@github.com/ungood/place",
            "https://github.com/Ungood/Place/",
            "git@GitHub.com:ungood/place.git",
            "https://user@github.com/ungood/place",
        ] {
            assert!(same(canonical, other), "{other}");
        }
    }

    #[test]
    fn different_repos_dont_match() {
        assert!(!same(
            "https://github.com/ungood/place.git",
            "https://github.com/nobody/place.git"
        ));
    }

    #[test]
    fn urls_and_paths() {
        assert!(is_url("https://github.com/ungood/place.git"));
        assert!(is_url("git@github.com:ungood/place.git"));
        assert!(!is_url("place"));
        assert!(!is_url("vendor/place"));
    }

    #[test]
    fn default_paths() {
        assert_eq!(
            default_path("https://github.com/ungood/place.git").as_deref(),
            Some("place")
        );
        assert_eq!(
            default_path("git@github.com:ungood/agents").as_deref(),
            Some("agents")
        );
        assert_eq!(
            default_path("https://github.com/ungood/place/").as_deref(),
            Some("place")
        );
        assert_eq!(default_path("https://github.com").as_deref(), None);
    }
}
