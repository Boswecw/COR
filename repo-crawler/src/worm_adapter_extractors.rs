
use serde_json::{json, Value};

fn edge(
    edge_id: &str,
    relation_type: &str,
    source_repo: &str,
    source_path: &str,
    discovery_method: &str,
    raw_reference: &str,
) -> Value {
    json!({
        "kind": "worm_edge",
        "schemaVersion": 1,
        "edgeId": edge_id,
        "relationType": relation_type,
        "sourceRepo": source_repo,
        "sourceArtifact": {
            "path": source_path
        },
        "discoveryMethod": discovery_method,
        "target": {
            "rawReference": raw_reference
        },
        "crawlScope": "cross_repo",
        "confidence": "medium",
        "posture": "evidence_bound",
        "timestamp": "2026-04-22T04:31:00-04:00"
    })
}

pub fn parse_gitmodules(source_repo: &str, source_path: &str, text: &str) -> Value {
    let mut urls = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("url = ") {
            let raw = rest.trim();
            if !raw.is_empty() {
                urls.push(raw.to_string());
            }
        }
    }

    let edges: Vec<Value> = urls
        .iter()
        .enumerate()
        .map(|(idx, raw)| {
            edge(
                &format!("gitmodules-edge-{}", idx + 1),
                "git_submodule",
                source_repo,
                source_path,
                "gitmodules_parse",
                raw,
            )
        })
        .collect();

    json!({
        "kind": "worm_adapter_emission",
        "schemaVersion": 1,
        "adapterName": "gitmodules_parse",
        "sourceRepo": source_repo,
        "sourceArtifact": {
            "path": source_path
        },
        "emittedEdges": edges,
        "skippedReferences": [],
        "posture": "evidence_bound",
        "timestamp": "2026-04-22T04:31:00-04:00"
    })
}

fn repo_style_dependency_value(raw: &str) -> bool {
    let value = raw.trim();
    value.contains("github.com/")
        || value.starts_with("git@github.com:")
        || value.starts_with("git+ssh://")
        || (
            value.contains('/')
            && !value.starts_with('^')
            && !value.starts_with('~')
            && !value.chars().all(|c| c.is_ascii_digit() || c == '.')
        )
}

pub fn parse_package_manifest(source_repo: &str, source_path: &str, text: &str) -> Result<Value, String> {
    let root: Value = serde_json::from_str(text)
        .map_err(|e| format!("failed to parse package manifest JSON: {}", e))?;

    let mut edges = Vec::new();
    let mut skipped = Vec::new();

    for section in ["dependencies", "devDependencies", "peerDependencies"] {
        let Some(map) = root.get(section).and_then(|v| v.as_object()) else {
            continue;
        };

        for (name, value) in map {
            let Some(raw) = value.as_str() else {
                skipped.push(json!({
                    "rawReference": name,
                    "reasonCode": "non_string_dependency_value"
                }));
                continue;
            };

            if repo_style_dependency_value(raw) {
                edges.push(edge(
                    &format!("package-edge-{}", edges.len() + 1),
                    "dependency_repo_reference",
                    source_repo,
                    source_path,
                    "package_manifest_parse",
                    raw,
                ));
            } else {
                skipped.push(json!({
                    "rawReference": raw,
                    "reasonCode": "non_repo_style_dependency_reference"
                }));
            }
        }
    }

    Ok(json!({
        "kind": "worm_adapter_emission",
        "schemaVersion": 1,
        "adapterName": "package_manifest_parse",
        "sourceRepo": source_repo,
        "sourceArtifact": {
            "path": source_path
        },
        "emittedEdges": edges,
        "skippedReferences": skipped,
        "posture": "evidence_bound",
        "timestamp": "2026-04-22T04:31:00-04:00"
    }))
}

fn maybe_push_cargo_edges(
    source_repo: &str,
    source_path: &str,
    source_table: &toml::Value,
    relation_type: &str,
    discovery_method: &str,
    edges: &mut Vec<Value>,
) {
    let Some(map) = source_table.as_table() else {
        return;
    };

    for (_name, value) in map {
        let Some(dep_table) = value.as_table() else {
            continue;
        };

        let Some(git_raw) = dep_table.get("git").and_then(|v| v.as_str()) else {
            continue;
        };

        edges.push(edge(
            &format!("cargo-edge-{}", edges.len() + 1),
            relation_type,
            source_repo,
            source_path,
            discovery_method,
            git_raw,
        ));
    }
}

pub fn parse_cargo_manifest(source_repo: &str, source_path: &str, text: &str) -> Result<Value, String> {
    let root: toml::Value = text
        .parse::<toml::Value>()
        .map_err(|e| format!("failed to parse Cargo.toml: {}", e))?;

    let mut edges = Vec::new();

    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(value) = root.get(section) {
            maybe_push_cargo_edges(
                source_repo,
                source_path,
                value,
                "dependency_repo_reference",
                "cargo_manifest_parse",
                &mut edges,
            );
        }
    }

    if let Some(workspace) = root.get("workspace").and_then(|v| v.as_table()) {
        if let Some(value) = workspace.get("dependencies") {
            maybe_push_cargo_edges(
                source_repo,
                source_path,
                value,
                "dependency_repo_reference",
                "cargo_workspace_manifest_parse",
                &mut edges,
            );
        }
    }

    Ok(json!({
        "kind": "worm_adapter_emission",
        "schemaVersion": 1,
        "adapterName": "cargo_manifest_parse",
        "sourceRepo": source_repo,
        "sourceArtifact": {
            "path": source_path
        },
        "emittedEdges": edges,
        "skippedReferences": [],
        "posture": "evidence_bound",
        "timestamp": "2026-04-22T04:31:00-04:00"
    }))
}

fn maybe_push_pyproject_dep_string(
    source_repo: &str,
    source_path: &str,
    raw: &str,
    discovery_method: &str,
    edges: &mut Vec<Value>,
) {
    let value = raw.trim();
    if value.contains(" @ git+")
        || value.starts_with("git+https://")
        || value.starts_with("git+ssh://")
        || value.contains("github.com/")
    {
        edges.push(edge(
            &format!("pyproject-edge-{}", edges.len() + 1),
            "dependency_repo_reference",
            source_repo,
            source_path,
            discovery_method,
            value,
        ));
    }
}

pub fn parse_pyproject_manifest(source_repo: &str, source_path: &str, text: &str) -> Result<Value, String> {
    let root: toml::Value = text
        .parse::<toml::Value>()
        .map_err(|e| format!("failed to parse pyproject.toml: {}", e))?;

    let mut edges = Vec::new();

    if let Some(project) = root.get("project").and_then(|v| v.as_table()) {
        if let Some(deps) = project.get("dependencies").and_then(|v| v.as_array()) {
            for dep in deps {
                if let Some(raw) = dep.as_str() {
                    maybe_push_pyproject_dep_string(
                        source_repo,
                        source_path,
                        raw,
                        "pyproject_pep621_parse",
                        &mut edges,
                    );
                }
            }
        }

        if let Some(optional) = project.get("optional-dependencies").and_then(|v| v.as_table()) {
            for (_group, deps) in optional {
                if let Some(dep_list) = deps.as_array() {
                    for dep in dep_list {
                        if let Some(raw) = dep.as_str() {
                            maybe_push_pyproject_dep_string(
                                source_repo,
                                source_path,
                                raw,
                                "pyproject_optional_dependencies_parse",
                                &mut edges,
                            );
                        }
                    }
                }
            }
        }
    }

    if let Some(tool) = root.get("tool").and_then(|v| v.as_table()) {
        if let Some(poetry) = tool.get("poetry").and_then(|v| v.as_table()) {
            if let Some(deps) = poetry.get("dependencies").and_then(|v| v.as_table()) {
                for (_name, value) in deps {
                    if let Some(raw) = value.as_str() {
                        maybe_push_pyproject_dep_string(
                            source_repo,
                            source_path,
                            raw,
                            "pyproject_poetry_parse",
                            &mut edges,
                        );
                    } else if let Some(dep_table) = value.as_table() {
                        if let Some(git_raw) = dep_table.get("git").and_then(|v| v.as_str()) {
                            maybe_push_pyproject_dep_string(
                                source_repo,
                                source_path,
                                git_raw,
                                "pyproject_poetry_git_parse",
                                &mut edges,
                            );
                        }
                    }
                }
            }

            if let Some(groups) = poetry.get("group").and_then(|v| v.as_table()) {
                for (_group_name, group_value) in groups {
                    let Some(group_table) = group_value.as_table() else {
                        continue;
                    };
                    let Some(deps) = group_table.get("dependencies").and_then(|v| v.as_table()) else {
                        continue;
                    };
                    for (_name, value) in deps {
                        if let Some(raw) = value.as_str() {
                            maybe_push_pyproject_dep_string(
                                source_repo,
                                source_path,
                                raw,
                                "pyproject_poetry_group_parse",
                                &mut edges,
                            );
                        } else if let Some(dep_table) = value.as_table() {
                            if let Some(git_raw) = dep_table.get("git").and_then(|v| v.as_str()) {
                                maybe_push_pyproject_dep_string(
                                    source_repo,
                                    source_path,
                                    git_raw,
                                    "pyproject_poetry_group_git_parse",
                                    &mut edges,
                                );
                            }
                        }
                    }
                }
            }
        }

        if let Some(uv) = tool.get("uv").and_then(|v| v.as_table()) {
            if let Some(sources) = uv.get("sources").and_then(|v| v.as_table()) {
                for (_name, source_value) in sources {
                    let Some(source_table) = source_value.as_table() else {
                        continue;
                    };
                    if let Some(git_raw) = source_table.get("git").and_then(|v| v.as_str()) {
                        maybe_push_pyproject_dep_string(
                            source_repo,
                            source_path,
                            git_raw,
                            "pyproject_uv_sources_parse",
                            &mut edges,
                        );
                    }
                }
            }
        }
    }

    Ok(json!({
        "kind": "worm_adapter_emission",
        "schemaVersion": 1,
        "adapterName": "pyproject_manifest_parse",
        "sourceRepo": source_repo,
        "sourceArtifact": {
            "path": source_path
        },
        "emittedEdges": edges,
        "skippedReferences": [],
        "posture": "evidence_bound",
        "timestamp": "2026-04-22T04:31:00-04:00"
    }))
}

pub fn parse_requirements_manifest(source_repo: &str, source_path: &str, text: &str) -> Result<Value, String> {
    let mut edges = Vec::new();
    let mut skipped = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with("-r ") || line.starts_with("--requirement ") {
            skipped.push(json!({
                "rawReference": line,
                "reasonCode": "nested_requirements_not_followed"
            }));
            continue;
        }

        let repo_ref = line.starts_with("git+https://")
            || line.starts_with("git+ssh://")
            || line.contains(" @ git+")
            || line.contains("github.com/");

        if repo_ref {
            edges.push(edge(
                &format!("requirements-edge-{}", edges.len() + 1),
                "dependency_repo_reference",
                source_repo,
                source_path,
                "requirements_manifest_parse",
                line,
            ));
        } else {
            skipped.push(json!({
                "rawReference": line,
                "reasonCode": "non_repo_style_dependency_reference"
            }));
        }
    }

    Ok(json!({
        "kind": "worm_adapter_emission",
        "schemaVersion": 1,
        "adapterName": "requirements_manifest_parse",
        "sourceRepo": source_repo,
        "sourceArtifact": {
            "path": source_path
        },
        "emittedEdges": edges,
        "skippedReferences": skipped,
        "posture": "evidence_bound",
        "timestamp": "2026-04-22T04:31:00-04:00"
    }))
}

pub fn parse_github_workflow(source_repo: &str, source_path: &str, text: &str) -> Result<Value, String> {
    let mut edges = Vec::new();
    let mut skipped = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();

        let candidate = if let Some(rest) = line.strip_prefix("uses:") {
            Some(rest.trim())
        } else if let Some(rest) = line.strip_prefix("- uses:") {
            Some(rest.trim())
        } else {
            None
        };

        let Some(raw) = candidate else {
            continue;
        };

        let raw = raw.trim_matches('"').trim_matches('\'');

        if raw.starts_with("./") {
            skipped.push(json!({
                "rawReference": raw,
                "reasonCode": "local_action_reference"
            }));
            continue;
        }
        if raw.starts_with("docker://") {
            skipped.push(json!({
                "rawReference": raw,
                "reasonCode": "docker_action_reference"
            }));
            continue;
        }
        if raw.contains('/') && raw.contains('@') {
            edges.push(edge(
                &format!("workflow-edge-{}", edges.len() + 1),
                "workflow_repo_reference",
                source_repo,
                source_path,
                "github_workflow_parse",
                raw,
            ));
        }
    }

    Ok(json!({
        "kind": "worm_adapter_emission",
        "schemaVersion": 1,
        "adapterName": "github_workflow_parse",
        "sourceRepo": source_repo,
        "sourceArtifact": {
            "path": source_path
        },
        "emittedEdges": edges,
        "skippedReferences": skipped,
        "posture": "evidence_bound",
        "timestamp": "2026-04-22T04:31:00-04:00"
    }))
}
