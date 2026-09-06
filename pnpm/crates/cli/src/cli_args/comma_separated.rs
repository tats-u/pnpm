use pnpm_resolving_parse_wanted_dependency::parse_wanted_dependency;
use std::path::{Path, PathBuf};

pub(crate) fn split_comma_separated(param: &str, base_dir: &Path) -> Vec<String> {
    let parsed = parse_wanted_dependency(param);
    let specifier = parsed.bare_specifier.as_deref().unwrap_or(param);
    if !specifier.contains(',') {
        return vec![param.to_string()];
    }
    if specifier.contains("://") {
        return vec![param.to_string()];
    }
    if refers_to_existing_local_path(specifier, base_dir) {
        return vec![param.to_string()];
    }
    param.split(',').map(str::trim).filter(|token| !token.is_empty()).map(str::to_string).collect()
}

pub(crate) fn split_comma_separated_selectors(
    selectors: &[String],
    base_dir: &Path,
) -> Vec<String> {
    selectors.iter().flat_map(|selector| split_comma_separated(selector, base_dir)).collect()
}

pub(crate) fn is_windows_drive_path(param: &str) -> bool {
    let bytes = param.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
}

fn refers_to_existing_local_path(param: &str, base_dir: &Path) -> bool {
    let path_part = if let Some(rest) = param.strip_prefix("file:") {
        rest
    } else if let Some(rest) = param.strip_prefix("link:") {
        rest
    } else if param.starts_with('.')
        || param.starts_with('/')
        || param.starts_with('~')
        || is_windows_drive_path(param)
    {
        param
    } else {
        return false;
    };
    let resolved = if Path::new(path_part).is_absolute() {
        PathBuf::from(path_part)
    } else {
        base_dir.join(path_part)
    };
    resolved.exists()
}
