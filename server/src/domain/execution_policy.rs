use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectExecutionPolicy {
    pub read_project: bool,
    pub write_roots: Vec<String>,
    pub reference_project_ids: Vec<String>,
    pub web_search: bool,
    pub shell_network: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DroidExecutionPolicyOverrides {
    #[serde(default)]
    pub read_project: Option<bool>,
    #[serde(default)]
    pub write_roots: Option<Vec<String>>,
    #[serde(default)]
    pub reference_project_ids: Option<Vec<String>>,
    #[serde(default)]
    pub web_search: Option<bool>,
    #[serde(default)]
    pub shell_network: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveExecutionPolicy {
    pub read_project: bool,
    pub write_roots: Vec<String>,
    pub reference_project_ids: Vec<String>,
    pub web_search: bool,
    pub shell_network: bool,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ExecutionPolicyError {
    #[error("write root must be a normalized relative path: {0}")]
    InvalidWriteRoot(String),
    #[error("reference project IDs must be nonempty and unique")]
    InvalidReferenceProjects,
    #[error("droid policy may only narrow its project policy")]
    DroidPolicyExceedsProject,
}

impl DroidExecutionPolicyOverrides {
    pub fn inherits_all(&self) -> bool {
        self.read_project.is_none()
            && self.write_roots.is_none()
            && self.reference_project_ids.is_none()
            && self.web_search.is_none()
            && self.shell_network.is_none()
    }
}

impl ProjectExecutionPolicy {
    pub fn validate(&self) -> Result<(), ExecutionPolicyError> {
        validate_write_roots(&self.write_roots)?;
        validate_reference_ids(&self.reference_project_ids)
    }

    pub fn effective_for(
        &self,
        overrides: &DroidExecutionPolicyOverrides,
    ) -> Result<EffectiveExecutionPolicy, ExecutionPolicyError> {
        self.validate()?;
        if let Some(roots) = overrides.write_roots.as_deref() {
            validate_write_roots(roots)?;
        }
        if let Some(ids) = overrides.reference_project_ids.as_deref() {
            validate_reference_ids(ids)?;
        }

        let write_roots = overrides
            .write_roots
            .as_ref()
            .unwrap_or(&self.write_roots)
            .clone();
        let reference_project_ids = overrides
            .reference_project_ids
            .as_ref()
            .unwrap_or(&self.reference_project_ids)
            .clone();
        if write_roots.iter().any(|root| {
            !self
                .write_roots
                .iter()
                .any(|allowed| path_is_within(root, allowed))
        }) || reference_project_ids
            .iter()
            .any(|id| !self.reference_project_ids.contains(id))
            || overrides.read_project == Some(true) && !self.read_project
            || overrides.web_search == Some(true) && !self.web_search
            || overrides.shell_network == Some(true) && !self.shell_network
        {
            return Err(ExecutionPolicyError::DroidPolicyExceedsProject);
        }

        Ok(EffectiveExecutionPolicy {
            read_project: self.read_project && overrides.read_project.unwrap_or(true),
            write_roots,
            reference_project_ids,
            web_search: self.web_search && overrides.web_search.unwrap_or(true),
            shell_network: self.shell_network && overrides.shell_network.unwrap_or(true),
        })
    }
}

impl EffectiveExecutionPolicy {
    pub fn allows_project_write(&self, path: &str) -> bool {
        self.write_roots
            .iter()
            .any(|root| path_is_within(path, root))
    }

    pub fn allows_reference_project(&self, project_id: &str) -> bool {
        self.reference_project_ids
            .iter()
            .any(|allowed| allowed == project_id)
    }
}

fn validate_write_roots(roots: &[String]) -> Result<(), ExecutionPolicyError> {
    let mut unique = std::collections::HashSet::new();
    for root in roots {
        let normalized = root.replace('\\', "/");
        let is_relative = !normalized.starts_with('/')
            && !normalized
                .as_bytes()
                .get(1)
                .is_some_and(|byte| *byte == b':');
        let has_empty_or_parent = normalized
            .split('/')
            .any(|segment| segment.is_empty() || segment == "..");
        if root.trim().is_empty()
            || normalized != *root
            || !is_relative
            || has_empty_or_parent
            || (root != "." && root.split('/').any(|segment| segment == "."))
            || !unique.insert(root)
        {
            return Err(ExecutionPolicyError::InvalidWriteRoot(root.clone()));
        }
    }
    Ok(())
}

fn validate_reference_ids(ids: &[String]) -> Result<(), ExecutionPolicyError> {
    let mut unique = std::collections::HashSet::new();
    if ids
        .iter()
        .any(|id| id.trim().is_empty() || !unique.insert(id))
    {
        return Err(ExecutionPolicyError::InvalidReferenceProjects);
    }
    Ok(())
}

fn path_is_within(path: &str, root: &str) -> bool {
    is_normalized_relative_path(path)
        && (root == "."
            || path == root
            || path
                .strip_prefix(root)
                .is_some_and(|tail| tail.starts_with('/')))
}

fn is_normalized_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !path.starts_with('/')
        && !path.as_bytes().get(1).is_some_and(|byte| *byte == b':')
        && (path == "."
            || path
                .split('/')
                .all(|segment| !segment.is_empty() && segment != "." && segment != ".."))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_policy() -> ProjectExecutionPolicy {
        ProjectExecutionPolicy {
            read_project: true,
            write_roots: vec!["src".into(), "docs".into()],
            reference_project_ids: vec!["proj-guides".into()],
            web_search: true,
            shell_network: false,
        }
    }

    #[test]
    fn droid_inherits_project_policy_and_can_only_narrow_it() {
        let effective = project_policy()
            .effective_for(&DroidExecutionPolicyOverrides {
                write_roots: Some(vec!["src/lib".into()]),
                web_search: Some(false),
                ..Default::default()
            })
            .unwrap();

        assert!(effective.read_project);
        assert!(effective.allows_project_write("src/lib/module.rs"));
        assert!(!effective.allows_project_write("src/main.rs"));
        assert!(!effective.web_search);
        assert!(!effective.shell_network);
        assert!(effective.allows_reference_project("proj-guides"));
        assert!(!effective.allows_reference_project("proj-other"));
    }

    #[test]
    fn droid_cannot_raise_project_capabilities_or_escape_write_roots() {
        assert_eq!(
            project_policy().effective_for(&DroidExecutionPolicyOverrides {
                shell_network: Some(true),
                ..Default::default()
            }),
            Err(ExecutionPolicyError::DroidPolicyExceedsProject)
        );
        assert_eq!(
            project_policy().effective_for(&DroidExecutionPolicyOverrides {
                write_roots: Some(vec!["src2".into()]),
                ..Default::default()
            }),
            Err(ExecutionPolicyError::DroidPolicyExceedsProject)
        );
    }

    #[test]
    fn project_policy_rejects_unsafe_or_ambiguous_roots() {
        for root in [
            "../outside",
            "/absolute",
            "C:/outside",
            "src//private",
            "src/../docs",
        ] {
            let policy = ProjectExecutionPolicy {
                write_roots: vec![root.into()],
                ..project_policy()
            };
            assert!(matches!(
                policy.validate(),
                Err(ExecutionPolicyError::InvalidWriteRoot(_))
            ));
        }
        assert!(path_is_within("src/file.rs", "src"));
        assert!(!path_is_within("src2/file.rs", "src"));
        assert!(!path_is_within("../outside", "."));
    }
}
