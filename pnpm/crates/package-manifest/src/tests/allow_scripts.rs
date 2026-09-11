use super::{HashMap, assert_eq, json, manifest_from_json, read_to_string, tempdir};
use crate::sync_allow_scripts_in_package_json;

#[test]
fn sync_does_not_create_missing_allow_scripts_or_replace_non_objects() {
    let dir = tempdir().unwrap();
    assert!(!sync_allow_scripts_in_package_json(dir.path(), [("esbuild", true)]).unwrap());
    assert!(!dir.path().join("package.json").exists());
    for manifest in [
        json!({}),
        json!({"allowScripts": null}),
        json!({"allowScripts": []}),
        json!({"allowScripts": true}),
    ] {
        let original = manifest.to_string();
        std::fs::write(dir.path().join("package.json"), &original).unwrap();
        assert!(!sync_allow_scripts_in_package_json(dir.path(), [("esbuild", true)]).unwrap());
        assert_eq!(read_to_string(dir.path().join("package.json")).unwrap(), original);
    }
}

#[test]
fn sync_updates_an_empty_allow_scripts_object_and_preserves_formatting() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("package.json");
    std::fs::write(&path, "{\n\t\"name\": \"example\",\n\t\"allowScripts\": {}\n}").unwrap();
    assert!(sync_allow_scripts_in_package_json(dir.path(), [("esbuild", false)]).unwrap());
    assert_eq!(
        read_to_string(&path).unwrap(),
        "{\n\t\"name\": \"example\",\n\t\"allowScripts\": {\n\t\t\"esbuild\": false\n\t}\n}"
    );
    assert!(!sync_allow_scripts_in_package_json(dir.path(), [("esbuild", false)]).unwrap());
}

#[test]
fn allow_scripts_reads_boolean_entries_only() {
    let (manifest, _dir) = manifest_from_json(json!({
        "allowScripts": {
            "esbuild": true,
            "sharp": "manual",
        },
    }));
    assert!(manifest.has_allow_scripts_object());
    assert_eq!(manifest.allow_scripts(), Some(HashMap::from([(String::from("esbuild"), true)])),);
}

#[test]
fn sync_allow_scripts_overlays_boolean_entries_without_touching_other_values() {
    let (mut manifest, _dir) = manifest_from_json(json!({
        "allowScripts": {
            "esbuild": false,
            "sharp": "manual",
        },
    }));
    assert!(manifest.sync_allow_scripts([("esbuild", true), ("rollup", false)]));
    assert_eq!(
        manifest.value().get("allowScripts"),
        Some(&json!({
            "esbuild": true,
            "rollup": false,
            "sharp": "manual",
        })),
    );
}

#[test]
fn sync_allow_scripts_in_package_json_updates_existing_allow_scripts_objects_only() {
    let dir = tempdir().unwrap();
    std::fs::write(
        dir.path().join("package.json"),
        serde_json::to_string_pretty(&json!({
            "allowScripts": {
                "esbuild": false,
            },
            "dependencies": {
                "foo": "1.0.0",
            },
        }))
        .unwrap(),
    )
    .unwrap();

    assert!(
        sync_allow_scripts_in_package_json(dir.path(), [("esbuild", true), ("rollup", false)])
            .expect("sync succeeds")
    );
    let saved: serde_json::Value =
        serde_json::from_str(&read_to_string(dir.path().join("package.json")).unwrap()).unwrap();
    assert_eq!(
        saved,
        json!({
            "allowScripts": {
                "esbuild": true,
                "rollup": false,
            },
            "dependencies": {
                "foo": "1.0.0",
            },
        }),
    );
}
