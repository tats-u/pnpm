use super::{INSTALL, INSTALL_MARKER, pacquet};
use assert_cmd::prelude::*;
use command_extra::CommandExtra;
use pnpm_testing_utils::bin::CommandTempCwd;
use std::fs;

#[test]
fn install_runs_a_build_approved_only_in_allow_scripts() {
    let harness = CommandTempCwd::init().add_mocked_registry();
    fs::write(
        harness.workspace.join("package.json"),
        serde_json::json!({
            "dependencies": {INSTALL: "1.0.0"},
            "allowScripts": {INSTALL: true},
        })
        .to_string(),
    )
    .unwrap();
    pacquet(&harness.workspace)
        .with_arg("install")
        .assert()
        .success();
    assert!(harness.workspace.join(INSTALL_MARKER).exists(), "the approved build ran");
}

#[test]
fn workspace_denial_overrides_allow_scripts_and_warns() {
    let harness = CommandTempCwd::init().add_mocked_registry();
    fs::write(
        harness.workspace.join("package.json"),
        serde_json::json!({
            "dependencies": {INSTALL: "1.0.0"},
            "allowScripts": {INSTALL: true},
        })
        .to_string(),
    )
    .unwrap();
    fs::write(
        harness.workspace.join("pnpm-workspace.yaml"),
        format!("allowBuilds:\n  '{INSTALL}': false\n"),
    )
    .unwrap();
    let assertion = pacquet(&harness.workspace)
        .with_arg("install")
        .assert()
        .success();
    let output = assertion.get_output();
    let logs = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(logs.contains("allowScripts") && logs.contains("conflicted"), "logs: {logs}");
    assert!(!harness.workspace.join(INSTALL_MARKER).exists(), "the denied build did not run");
}

#[test]
fn add_allow_build_syncs_both_manifest_policies() {
    let harness = CommandTempCwd::init().add_mocked_registry();
    fs::write(harness.workspace.join("package.json"), r#"{"allowScripts":{"preserved":false}}"#)
        .unwrap();
    pacquet(&harness.workspace)
        .with_args(["add", INSTALL, "--allow-build", INSTALL])
        .assert()
        .success();
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(harness.workspace.join("package.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["allowScripts"], serde_json::json!({INSTALL: true, "preserved": false}));
    let yaml = fs::read_to_string(harness.workspace.join("pnpm-workspace.yaml")).unwrap();
    assert!(yaml.contains("preserved: false") && yaml.contains(INSTALL), "yaml: {yaml}");
    assert!(harness.workspace.join(INSTALL_MARKER).exists(), "the approved build ran");
}

#[cfg(unix)]
#[test]
fn ignored_build_scaffold_syncs_boolean_decisions_but_not_placeholders() {
    let (harness, workspace) = super::workspace_with_unapproved_build();
    let path = workspace.join("package.json");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    manifest["allowScripts"] = serde_json::json!({"from-package-json": false});
    fs::write(&path, manifest.to_string()).unwrap();
    let yaml_path = workspace.join("pnpm-workspace.yaml");
    let settings = fs::read_to_string(&yaml_path).unwrap();
    fs::write(&yaml_path, format!("{settings}allowBuilds:\n  from-workspace: true\n")).unwrap();

    super::pacquet_in_terminal(&workspace)
        .with_arg("install")
        .assert()
        .success();

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        manifest["allowScripts"],
        serde_json::json!({"from-package-json": false, "from-workspace": true})
    );
    let yaml = fs::read_to_string(yaml_path).unwrap();
    assert!(yaml.contains("from-package-json: false"), "yaml: {yaml}");
    assert!(yaml.contains("set this to true or false"), "yaml: {yaml}");
    assert!(!workspace.join(INSTALL_MARKER).exists(), "the undecided build did not run");
    drop(harness);
}
