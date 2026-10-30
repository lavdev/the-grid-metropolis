use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use serde_json::to_string;
use thiserror::Error;

pub use crate::domain::execution_policy::{
    DroidExecutionPolicyOverrides, EffectiveExecutionPolicy, ExecutionPolicyError,
    ProjectExecutionPolicy,
};

const CODEX_PROFILE_NAME: &str = "grid_metropolis_run";

#[derive(Debug, Error)]
pub enum CodexProfileError {
    #[error("project root is unavailable or is not a directory")]
    InvalidProjectRoot,
    #[error("reference project root is unavailable or is not a directory: {0}")]
    InvalidReferenceRoot(String),
    #[error("write root escapes the canonical project root: {0}")]
    WriteRootEscapesProject(String),
    #[error("policy path cannot be represented as a Unicode app-server path")]
    NonUnicodePath,
    #[error("could not serialize a Codex configuration override")]
    ConfigSerialization,
}

/// Build CLI configuration overrides for a fresh app-server process.
/// Each run uses its own server process, so this profile cannot leak to another Droid.
/// The filesystem starts denied and only receives the effective project/reference grants.
pub fn codex_profile_config_args(
    project_root: Option<&Path>,
    reference_project_roots: &[PathBuf],
    policy: &EffectiveExecutionPolicy,
) -> Result<Vec<OsString>, CodexProfileError> {
    let canonical_project = project_root.map(canonical_project_root).transpose()?;
    let mut filesystem_grants = BTreeMap::new();

    if policy.read_project {
        let root = canonical_project
            .as_ref()
            .ok_or(CodexProfileError::InvalidProjectRoot)?;
        filesystem_grants.insert(root.clone(), "read");
    }
    if !policy.write_roots.is_empty() {
        let root = canonical_project
            .as_ref()
            .ok_or(CodexProfileError::InvalidProjectRoot)?;
        for write_root in &policy.write_roots {
            let absolute = resolve_write_root(root, write_root)?;
            filesystem_grants.insert(absolute, "write");
        }
    }
    for reference_root in reference_project_roots {
        let canonical = canonical_directory(reference_root).ok_or_else(|| {
            CodexProfileError::InvalidReferenceRoot(reference_root.display().to_string())
        })?;
        if canonical_project.as_ref() == Some(&canonical) {
            return Err(CodexProfileError::InvalidReferenceRoot(
                reference_root.display().to_string(),
            ));
        }
        filesystem_grants.insert(canonical, "read");
    }

    let mut args = Vec::new();
    push_config(
        &mut args,
        "default_permissions",
        &quoted(CODEX_PROFILE_NAME)?,
    );
    push_config(
        &mut args,
        &format!("permissions.{CODEX_PROFILE_NAME}.extends"),
        &quoted(":read-only")?,
    );
    let mut filesystem_config = vec![(":root".to_owned(), "deny"), (":minimal".into(), "read")];
    for (path, access) in filesystem_grants {
        let path = path.to_str().ok_or(CodexProfileError::NonUnicodePath)?;
        filesystem_config.push((path.to_owned(), access));
    }
    push_config(
        &mut args,
        &format!("permissions.{CODEX_PROFILE_NAME}.filesystem"),
        &inline_string_table(&filesystem_config)?,
    );

    push_config(
        &mut args,
        &format!("permissions.{CODEX_PROFILE_NAME}.network.enabled"),
        if policy.shell_network {
            "true"
        } else {
            "false"
        },
    );
    push_config(&mut args, "features.network_proxy", "true");
    push_config(&mut args, "features.prefer_mxc", "true");
    if policy.shell_network {
        push_config(
            &mut args,
            &format!("permissions.{CODEX_PROFILE_NAME}.network.domains"),
            &inline_string_table(&[("*".into(), "allow")])?,
        );
    }
    push_config(
        &mut args,
        "web_search",
        &quoted(if policy.web_search {
            "cached"
        } else {
            "disabled"
        })?,
    );
    Ok(args)
}

fn push_config(args: &mut Vec<OsString>, key: &str, value: &str) {
    args.push(OsString::from("--config"));
    args.push(OsString::from(format!("{key}={value}")));
}

fn quoted(value: &str) -> Result<String, CodexProfileError> {
    to_string(value).map_err(|_| CodexProfileError::ConfigSerialization)
}

fn inline_string_table(values: &[(String, &str)]) -> Result<String, CodexProfileError> {
    let fields = values
        .iter()
        .map(|(key, value)| Ok(format!("{}={}", quoted(key)?, quoted(value)?)))
        .collect::<Result<Vec<_>, CodexProfileError>>()?;
    Ok(format!("{{{}}}", fields.join(",")))
}

fn canonical_project_root(path: &Path) -> Result<PathBuf, CodexProfileError> {
    canonical_directory(path).ok_or(CodexProfileError::InvalidProjectRoot)
}

fn canonical_directory(path: &Path) -> Option<PathBuf> {
    let canonical = fs::canonicalize(path).ok()?;
    canonical.is_dir().then_some(canonical)
}

fn resolve_write_root(
    project_root: &Path,
    relative_root: &str,
) -> Result<PathBuf, CodexProfileError> {
    if !policy_relative_path_is_valid(relative_root) {
        return Err(CodexProfileError::WriteRootEscapesProject(
            relative_root.to_owned(),
        ));
    }
    if relative_root == "." {
        return Ok(project_root.to_path_buf());
    }

    let candidate = project_root.join(relative_root);
    let mut nearest_existing = candidate.as_path();
    let mut missing_components = Vec::new();
    while !nearest_existing.exists() {
        let Some(component) = nearest_existing.file_name() else {
            return Err(CodexProfileError::WriteRootEscapesProject(
                relative_root.to_owned(),
            ));
        };
        missing_components.push(component.to_os_string());
        nearest_existing = nearest_existing
            .parent()
            .ok_or_else(|| CodexProfileError::WriteRootEscapesProject(relative_root.to_owned()))?;
    }

    let canonical_existing = fs::canonicalize(nearest_existing)
        .map_err(|_| CodexProfileError::WriteRootEscapesProject(relative_root.to_owned()))?;
    if !canonical_existing.starts_with(project_root) {
        return Err(CodexProfileError::WriteRootEscapesProject(
            relative_root.to_owned(),
        ));
    }
    let mut resolved = canonical_existing;
    for component in missing_components.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn policy_relative_path_is_valid(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !path.starts_with('/')
        && !path.as_bytes().get(1).is_some_and(|byte| *byte == b':')
        && path
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

/// Resolve the effective droid policy as a subset of the project ceiling.
pub fn resolve_droid_policy(
    project: &ProjectExecutionPolicy,
    droid: &DroidExecutionPolicyOverrides,
) -> Result<EffectiveExecutionPolicy, ExecutionPolicyError> {
    project.effective_for(droid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn codex_profile_config_is_default_deny_and_maps_separate_capabilities() {
        let temp_root = std::env::temp_dir();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let fixture = temp_root.join(format!("grid-policy-config-{stamp}"));
        let project_root = fixture.join("project");
        let writable_root = project_root.join("src").join("generated");
        let reference_root = fixture.join("reference");
        fs::create_dir_all(&writable_root).unwrap();
        fs::create_dir_all(&reference_root).unwrap();
        let policy = EffectiveExecutionPolicy {
            read_project: true,
            write_roots: vec!["src/generated".into()],
            reference_project_ids: vec!["reference-project".into()],
            web_search: true,
            shell_network: false,
        };

        let args = codex_profile_config_args(
            Some(&project_root),
            std::slice::from_ref(&reference_root),
            &policy,
        )
        .unwrap();
        let args = args
            .iter()
            .map(|value| value.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let config_values = args
            .windows(2)
            .filter_map(|pair| (pair[0] == "--config").then_some(pair[1].as_str()))
            .collect::<Vec<_>>();
        let filesystem = config_values
            .iter()
            .find(|value| value.starts_with("permissions.grid_metropolis_run.filesystem="))
            .unwrap();
        let canonical_project = fs::canonicalize(&project_root).unwrap();
        let canonical_write = fs::canonicalize(&writable_root).unwrap();
        let canonical_reference = fs::canonicalize(&reference_root).unwrap();
        assert!(filesystem.contains("\":root\"=\"deny\""));
        assert!(filesystem.contains("\":minimal\"=\"read\""));
        assert!(filesystem.contains(&format!(
            "{}=\"read\"",
            quoted(canonical_project.to_str().unwrap()).unwrap()
        )));
        assert!(filesystem.contains(&format!(
            "{}=\"write\"",
            quoted(canonical_write.to_str().unwrap()).unwrap()
        )));
        assert!(filesystem.contains(&format!(
            "{}=\"read\"",
            quoted(canonical_reference.to_str().unwrap()).unwrap()
        )));
        assert!(config_values.contains(&"permissions.grid_metropolis_run.network.enabled=false"));
        assert!(config_values.contains(&"web_search=\"cached\""));
        assert!(config_values.contains(&"features.prefer_mxc=true"));
        assert!(
            !config_values
                .iter()
                .any(|value| value.contains("network.domains"))
        );

        assert!(fixture.starts_with(&temp_root));
        fs::remove_dir_all(fixture).unwrap();
    }
}
