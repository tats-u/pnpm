use crate::{boolean_negations::with_boolean_negations, cli_args::CliArgs, prepare_cli_argv};
use clap::CommandFactory;
use clap::error::ErrorKind;
use pretty_assertions::assert_eq;
use std::ffi::OsString;

fn prepare(tokens: &[&str]) -> Result<Vec<String>, clap::Error> {
    prepare_cli_argv(
        tokens
            .iter()
            .map(OsString::from)
            .collect(),
    )
    .map(|(_, argv)| {
        argv.into_iter()
            .map(|token| token.into_string().expect("test tokens are UTF-8"))
            .collect()
    })
}

#[test]
fn add_short_t_is_rejected_with_long_spelling_guidance() {
    let err = prepare(&["pnpm", "add", "foo", "-T"]).expect_err("`add -T` should fail");
    assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    let message = err.to_string();
    assert!(message.contains("-T"), "{message}");
    assert!(message.contains("--save-types"), "{message}");
    assert!(message.contains("--tilde"), "{message}");
}

#[test]
fn a_value_spelled_short_t_is_left_alone() {
    assert_eq!(
        prepare(&["pnpm", "add", "foo", "--save-prefix", "-T"])
            .expect("value spelling stays a value"),
        ["pnpm", "add", "foo", "--save-prefix", "-T"],
    );
}

#[test]
fn tokens_after_the_separator_are_left_alone() {
    assert_eq!(
        prepare(&["pnpm", "add", "foo", "--", "-T"]).expect("separator stops the scan"),
        ["pnpm", "add", "foo", "--", "-T"],
    );
}

#[test]
fn add_help_lists_only_the_long_options() {
    let mut command = with_boolean_negations(CliArgs::command());
    let add = command.find_subcommand_mut("add").expect("`add` subcommand exists");
    let mut help = Vec::new();
    add.write_long_help(&mut help).expect("render add help");
    let help = String::from_utf8(help).expect("help is valid UTF-8");

    assert!(help.contains("--save-types"), "{help}");
    assert!(help.contains("--tilde"), "{help}");
    assert!(!help.contains("-T, --save-types"), "{help}");
    assert!(!help.contains("-T, --tilde"), "{help}");
}
