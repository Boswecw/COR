
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalIdentity {
    pub host: String,
    pub owner: String,
    pub repo: String,
    pub display: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationResult {
    pub posture: String,
    pub method: String,
    pub canonical: Option<CanonicalIdentity>,
}

fn trim_git_suffix(input: &str) -> &str {
    input.strip_suffix(".git").unwrap_or(input)
}

fn parse_owner_repo(input: &str) -> Option<(String, String)> {
    let candidate = trim_git_suffix(input.trim().trim_matches('/'));
    let mut parts = candidate.split('/');

    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim();

    if owner.is_empty() || repo.is_empty() || parts.next().is_some() {
        return None;
    }

    Some((owner.to_string(), repo.to_string()))
}

fn resolved(method: &str, host: &str, owner: &str, repo: &str) -> NormalizationResult {
    let display = format!("{owner}/{repo}");
    NormalizationResult {
        posture: "resolved".to_string(),
        method: method.to_string(),
        canonical: Some(CanonicalIdentity {
            host: host.to_string(),
            owner: owner.to_string(),
            repo: repo.to_string(),
            display,
        }),
    }
}

pub fn normalize_reference(raw: &str) -> NormalizationResult {
    let raw = raw.trim();

    if let Some(rest) = raw.strip_prefix("git+ssh://git@github.com/") {
        if let Some((owner, repo)) = parse_owner_repo(rest) {
            return resolved("git_ssh_remote_parse", "github.com", &owner, &repo);
        }
    }

    if let Some(rest) = raw.strip_prefix("git@github.com:") {
        if let Some((owner, repo)) = parse_owner_repo(rest) {
            return resolved("ssh_remote_parse", "github.com", &owner, &repo);
        }
    }

    if let Some(rest) = raw.strip_prefix("https://github.com/") {
        if let Some((owner, repo)) = parse_owner_repo(rest) {
            return resolved("https_remote_parse", "github.com", &owner, &repo);
        }
    }

    if raw.starts_with("../") || raw.starts_with("./") {
        return NormalizationResult {
            posture: "ambiguous".to_string(),
            method: "relative_path_inference".to_string(),
            canonical: None,
        };
    }

    if let Some((owner, repo)) = parse_owner_repo(raw) {
        return resolved("owner_repo_direct", "github.com", &owner, &repo);
    }

    NormalizationResult {
        posture: "unresolved".to_string(),
        method: "none".to_string(),
        canonical: None,
    }
}
