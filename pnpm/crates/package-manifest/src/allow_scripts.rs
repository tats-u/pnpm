use super::{PackageManifest, PackageManifestError, Value};
use std::path::Path;

impl PackageManifest {
    #[must_use]
    pub fn has_allow_scripts_object(&self) -> bool {
        self.value.get("allowScripts").is_some_and(Value::is_object)
    }

    #[must_use]
    pub fn allow_scripts(&self) -> Option<std::collections::HashMap<String, bool>> {
        self.value
            .get("allowScripts")
            .and_then(Value::as_object)
            .map(|allow_scripts| {
                allow_scripts
                    .iter()
                    .filter_map(|(name, value)| {
                        value
                            .as_bool()
                            .map(|allowed| (name.clone(), allowed))
                    })
                    .collect()
            })
    }

    /// Overlay boolean `allowScripts` entries on the existing top-level
    /// `allowScripts` object, preserving any non-boolean entries the caller is
    /// not rewriting. Returns whether the manifest changed. Does nothing when
    /// the field is absent or not an object.
    pub fn sync_allow_scripts<'a, Entries>(&mut self, entries: Entries) -> bool
    where
        Entries: IntoIterator<Item = (&'a str, bool)>,
    {
        let Some(manifest) = self.value.as_object_mut() else {
            return false;
        };
        let Some(allow_scripts) = manifest.get_mut("allowScripts").and_then(Value::as_object_mut)
        else {
            return false;
        };
        let mut changed = false;
        for (name, allowed) in entries {
            let value = Value::Bool(allowed);
            if allow_scripts.get(name) != Some(&value) {
                allow_scripts.insert(name.to_string(), value);
                changed = true;
            }
        }
        changed
    }
}

/// Sync boolean entries into the top-level `allowScripts` object of
/// `<dir>/package.json`, preserving unrelated fields and entries not supplied
/// by the caller. Returns `true` when the file was updated, `false`
/// when there is no `package.json`, no `allowScripts` object, or the file
/// already encoded the requested values.
pub fn sync_allow_scripts_in_package_json<'a, Entries>(
    dir: &Path,
    entries: Entries,
) -> Result<bool, PackageManifestError>
where
    Entries: IntoIterator<Item = (&'a str, bool)>,
{
    let manifest_path = dir.join("package.json");
    let mut manifest = match PackageManifest::from_path(manifest_path) {
        Ok(manifest) => manifest,
        Err(PackageManifestError::NoImporterManifestFound(_)) => {
            return Ok(false);
        }
        Err(err) => return Err(err),
    };
    if !manifest.has_allow_scripts_object() || !manifest.sync_allow_scripts(entries) {
        return Ok(false);
    }
    manifest.save()?;
    Ok(true)
}
