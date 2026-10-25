use super::{
    Config, EnvVar, EnvVarOs, GetCurrentDir, GetHomeDir, LinkProbe, OsString, Path, PathBuf,
    capture_warnings, fs, io, tempdir,
};
use std::collections::HashMap;

#[test]
fn package_json_allow_scripts_are_used_when_allow_builds_are_absent() {
    fake_env!(load_with_fake_env);
    set_fake_env(&[]);
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "allowScripts": {
                "esbuild": true,
            },
        }))
        .unwrap(),
    )
    .unwrap();

    let config = load_with_fake_env(dir.path());
    assert_eq!(config.allow_builds, HashMap::from([(String::from("esbuild"), true)]));
}

#[test]
fn workspace_allow_builds_override_package_json_allow_scripts_with_warning() {
    fake_env!(load_with_fake_env);
    set_fake_env(&[]);
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "allowScripts": {
                "esbuild": false,
                "sharp": true,
            },
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(dir.path().join("pnpm-workspace.yaml"), "allowBuilds:\n  esbuild: true\n").unwrap();

    let warnings = capture_warnings(|| {
        let config = load_with_fake_env(dir.path());
        assert_eq!(
            config.allow_builds,
            HashMap::from([(String::from("esbuild"), true), (String::from("sharp"), true),]),
        );
    });
    assert!(
        warnings
            .iter()
            .any(|warning| {
                warning.contains(r#"package.json "allowScripts" entries conflicted"#)
                    && warning.contains("esbuild")
            })
    );
}

#[test]
fn workspace_permissions_override_allow_scripts_from_a_workspace_child() {
    fake_env!(load_with_fake_env);
    set_fake_env(&[]);
    let dir = tempdir().unwrap();
    let child = dir.path().join("packages/child");
    fs::create_dir_all(&child).unwrap();
    fs::write(child.join("package.json"), r#"{"allowScripts":{"child-only":true}}"#).unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"allowScripts":{"esbuild":true,"sharp":false,"root-only":true}}"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("pnpm-workspace.yaml"),
        "packages:\n  - packages/*\npermissions:\n  esbuild:\n    build: false\n  sharp:\n    build: true\n",
    ).unwrap();

    let warnings = capture_warnings(|| {
        let config = load_with_fake_env(&child);
        assert_eq!(
            config.allow_builds,
            HashMap::from([
                (String::from("esbuild"), false),
                (String::from("sharp"), true),
                (String::from("root-only"), true),
            ])
        );
    });
    dbg!(&warnings);
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("permissions") && warning.contains("esbuild, sharp"))
    );
}

#[test]
fn an_absent_package_json_does_not_prevent_loading_workspace_settings() {
    fake_env!(load_with_fake_env);
    set_fake_env(&[]);
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("pnpm-workspace.yaml"), "allowBuilds:\n  esbuild: false\n").unwrap();
    assert_eq!(
        load_with_fake_env(dir.path()).allow_builds,
        HashMap::from([(String::from("esbuild"), false)])
    );
}
