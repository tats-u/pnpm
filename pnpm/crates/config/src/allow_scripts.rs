use crate::{Config, LoadWorkspaceYamlError};
use pnpm_package_manifest::PackageManifest;
use std::{collections::HashMap, path::Path};

pub(super) fn apply_root_allow_scripts(
    config: &mut Config,
    root_project_manifest_dir: &Path,
) -> Result<(), LoadWorkspaceYamlError> {
    let manifest = match PackageManifest::from_path(root_project_manifest_dir.join("package.json"))
    {
        Ok(manifest) => manifest,
        Err(pnpm_package_manifest::PackageManifestError::NoImporterManifestFound(_)) => {
            return Ok(());
        }
        Err(err) => {
            return Err(LoadWorkspaceYamlError::ReadRootManifest { source: Box::new(err) });
        }
    };
    let Some(allow_scripts) = manifest.allow_scripts() else {
        return Ok(());
    };
    let conflicting_entries =
        merge_allow_scripts_into_allow_builds(&mut config.allow_builds, allow_scripts);
    config.allow_scripts_warning = (!conflicting_entries.is_empty()).then(|| format!(
        r#"package.json "allowScripts" entries conflicted with "allowBuilds" or "permissions" in pnpm-workspace.yaml and were ignored; the workspace values were used: {}"#,
        conflicting_entries.join(", "),
    ));
    if let Some(warning) = &config.allow_scripts_warning {
        tracing::warn!(
            target: "pacquet::config",
            entries = %conflicting_entries.join(", "),
            "{warning}",
        );
    }
    Ok(())
}

fn merge_allow_scripts_into_allow_builds(
    allow_builds: &mut HashMap<String, bool>,
    allow_scripts: HashMap<String, bool>,
) -> Vec<String> {
    let mut conflicting_entries = Vec::new();
    for (name, allowed) in allow_scripts {
        match allow_builds.get(&name) {
            Some(existing) if *existing != allowed => conflicting_entries.push(name),
            Some(_) => {}
            None => {
                allow_builds.insert(name, allowed);
            }
        }
    }
    conflicting_entries.sort_unstable();
    conflicting_entries
}
