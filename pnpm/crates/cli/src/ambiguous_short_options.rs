//! Reject short options that pacquet keeps ambiguous on purpose.
//!
//! `pnpm add` has both `--save-types` and `--tilde`, so a bare `-T` needs a
//! diagnostic that points at the long spellings instead of guessing.

use crate::{
    flag_relocation::{ArgTable, find_positional, token_width},
    parse_boundary,
};
use clap::{Command, error::ErrorKind};
use std::ffi::OsString;

const AMBIGUOUS_SHORT_OPTION: char = 'T';
const AMBIGUOUS_SHORT_TOKEN: &str = "-T";

pub(crate) fn reject_for_add(
    cmd: &Command,
    argv: Vec<OsString>,
) -> Result<Vec<OsString>, clap::Error> {
    let top_level = ArgTable::top_level(cmd);
    let subcommand_union = ArgTable::subcommand_union(cmd);
    let Some(subcommand_index) = find_positional(&argv, 1, &top_level, &subcommand_union) else {
        return Ok(argv);
    };
    let names_add = cmd
        .find_subcommand(&argv[subcommand_index])
        .is_some_and(|subcommand| subcommand.get_name() == "add");
    if !names_add {
        return Ok(argv);
    }

    let passthrough_from = parse_boundary::passthrough_from(&argv).unwrap_or(argv.len());
    let mut index = subcommand_index + 1;
    while index < passthrough_from.min(argv.len()) {
        let Some(token) = argv[index].to_str() else {
            index += 1;
            continue;
        };
        if token == "--" {
            break;
        }
        if short_cluster_contains_ambiguous_t(token, &top_level, &subcommand_union) {
            return Err(cmd.clone().error(
                ErrorKind::UnknownArgument,
                format!(
                    "short option '{AMBIGUOUS_SHORT_TOKEN}' is ambiguous, use '--save-types' or '--tilde'",
                ),
            ));
        }
        index += token_span(token, &top_level, &subcommand_union);
    }
    Ok(argv)
}

fn short_cluster_contains_ambiguous_t(
    token: &str,
    top_level: &ArgTable,
    subcommand_union: &ArgTable,
) -> bool {
    let Some(cluster) = token
        .strip_prefix('-')
        .filter(|rest| !rest.is_empty() && !rest.starts_with('-'))
    else {
        return false;
    };
    for short in cluster.chars() {
        if short == AMBIGUOUS_SHORT_OPTION {
            return true;
        }
        let consumes_value = top_level
            .short_consumes_value(short)
            .or_else(|| subcommand_union.short_consumes_value(short))
            .unwrap_or(false);
        if consumes_value {
            return false;
        }
    }
    false
}

fn token_span(token: &str, top_level: &ArgTable, subcommand_union: &ArgTable) -> usize {
    let Some(rest) = token
        .strip_prefix('-')
        .filter(|rest| !rest.is_empty())
    else {
        return 1;
    };
    if let Some(long) = rest.strip_prefix('-') {
        let (name, has_inline_value) = long
            .split_once('=')
            .map_or((long, false), |(name, _)| (name, true));
        let consumes_value = top_level
            .long_consumes_value(name)
            .or_else(|| subcommand_union.long_consumes_value(name))
            .unwrap_or(false);
        return token_width(consumes_value, has_inline_value);
    }
    let short = rest
        .chars()
        .next()
        .expect("checked non-empty");
    let is_bare_short = rest.chars().count() == 1;
    let consumes_value = top_level
        .short_consumes_value(short)
        .or_else(|| subcommand_union.short_consumes_value(short))
        .unwrap_or(false);
    token_width(consumes_value && is_bare_short, false)
}

#[cfg(test)]
mod tests;
