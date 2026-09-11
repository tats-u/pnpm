use super::{ApprovalDecision, ApproveBuildsError, partition_params, sort_unique, write_approval_settings};
use std::collections::{BTreeMap, HashMap};

fn pending(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_string()).collect()
}

fn params(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| (*arg).to_string()).collect()
}

#[test]
fn splits_approved_and_denied() {
    let (approved, denied) =
        partition_params(&params(&["foo", "!bar"]), &pending(&["foo", "bar"])).unwrap();
    assert_eq!(approved, vec!["foo".to_string()]);
    assert_eq!(denied, vec!["bar".to_string()]);
}

// Ports pnpm's `positional arguments with unknown package throws error`.
#[test]
fn rejects_unknown_approved_package() {
    let err = partition_params(&params(&["nope"]), &pending(&["foo"])).unwrap_err();
    let ApproveBuildsError::UnknownPackages(names) = err else {
        panic!("expected UnknownPackages, got {err:?}");
    };
    assert_eq!(names, vec!["nope".to_string()]);
}

// Ports pnpm's `!pkg with unknown package throws error`.
#[test]
fn rejects_unknown_denied_package() {
    let err = partition_params(&params(&["!nope"]), &pending(&["foo"])).unwrap_err();
    let ApproveBuildsError::UnknownPackages(names) = err else {
        panic!("expected UnknownPackages, got {err:?}");
    };
    assert_eq!(names, vec!["nope".to_string()]);
}

// Ports pnpm's `contradictory arguments throw error`.
#[test]
fn rejects_contradictory_arguments() {
    let err = partition_params(&params(&["foo", "!foo"]), &pending(&["foo"])).unwrap_err();
    let ApproveBuildsError::ContradictingArgs(names) = err else {
        panic!("expected ContradictingArgs, got {err:?}");
    };
    assert_eq!(names, vec!["foo".to_string()]);
}

#[test]
fn sort_unique_dedupes_and_sorts() {
    assert_eq!(sort_unique(params(&["b", "a", "b"])), vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn write_approval_settings_syncs_existing_package_json_allow_scripts() {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(
        dir.path().join("package.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "allowScripts": {
                "sharp": true,
            },
        }))
        .unwrap(),
    )
    .expect("package.json written");

    write_approval_settings(
        dir.path(),
        dir.path(),
        &HashMap::from([(String::from("sharp"), true)]),
        &ApprovalDecision {
            build_packages: vec![String::from("esbuild")],
            decisions: BTreeMap::from([(String::from("esbuild"), true)]),
            clear_all: false,
        },
    )
    .expect("settings written");

    let workspace_yaml = std::fs::read_to_string(dir.path().join("pnpm-workspace.yaml")).unwrap();
    assert!(workspace_yaml.contains("esbuild: true"));
    assert!(workspace_yaml.contains("sharp: true"));

    let package_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join("package.json")).unwrap())
            .unwrap();
    assert_eq!(
        package_json,
        serde_json::json!({
            "allowScripts": {
                "esbuild": true,
                "sharp": true,
            },
        }),
    );
}
