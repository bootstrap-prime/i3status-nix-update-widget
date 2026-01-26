use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

const BOOTED_SYSTEM: &str = "/run/booted-system";
const CURRENT_SYSTEM: &str = "/run/current-system";

pub struct VersionMismatch {
    pub name: String,
    pub booted: String,
    pub current: String,
}

pub fn check_reboot_needed() -> Result<Vec<VersionMismatch>> {
    let booted_versions = get_all_versions(BOOTED_SYSTEM)?;
    let current_versions = get_all_versions(CURRENT_SYSTEM)?;

    let mut mismatches = Vec::new();

    for (name, booted_ver) in &booted_versions {
        if let Some(current_ver) = current_versions.get(name) {
            if booted_ver != current_ver {
                mismatches.push(VersionMismatch {
                    name: name.clone(),
                    booted: booted_ver.clone(),
                    current: current_ver.clone(),
                });
            }
        }
    }

    for (name, current_ver) in &current_versions {
        if !booted_versions.contains_key(name) {
            mismatches.push(VersionMismatch {
                name: name.clone(),
                booted: "(none)".to_string(),
                current: current_ver.clone(),
            });
        }
    }

    Ok(mismatches)
}

fn get_all_versions(system_path: &str) -> Result<HashMap<String, String>> {
    let mut versions = HashMap::new();

    let kernel_modules_path = format!("{}/kernel-modules/lib/modules", system_path);
    let modules_dir = Path::new(&kernel_modules_path);

    if !modules_dir.exists() {
        return Ok(versions);
    }

    if let Some(kernel_ver) = get_kernel_version(modules_dir)? {
        versions.insert("kernel".to_string(), kernel_ver.clone());

        let ver_path = modules_dir.join(&kernel_ver);
        let oot_modules = get_oot_module_versions(&ver_path)?;
        versions.extend(oot_modules);
    }

    Ok(versions)
}

fn get_kernel_version(modules_dir: &Path) -> Result<Option<String>> {
    for entry in fs::read_dir(modules_dir).context("Failed to read modules directory")? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(|c: char| c.is_ascii_digit()) {
            return Ok(Some(name));
        }
    }
    Ok(None)
}

fn get_oot_module_versions(ver_path: &Path) -> Result<HashMap<String, String>> {
    let mut versions = HashMap::new();

    // misc/ has nvidia, updates/ has xone, kernel/drivers/* has other oot modules
    for dir_name in ["misc", "updates"] {
        let dir_path = ver_path.join(dir_name);
        if dir_path.is_symlink() || dir_path.is_dir() {
            if let Some((name, version)) = get_module_version(&dir_path)? {
                versions.insert(name, version);
            }
        }
    }

    let drivers_path = ver_path.join("kernel/drivers");
    if drivers_path.exists() {
        for entry in fs::read_dir(&drivers_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_symlink() {
                let target = fs::read_link(&path)?;
                let target_str = target.to_string_lossy();
                // skip in-tree kernel modules
                if !target_str.contains("linux-") || !target_str.contains("-modules") {
                    if let Some((name, version)) = get_module_version(&path)? {
                        versions.insert(name, version);
                    }
                }
            }
        }
    }

    Ok(versions)
}

// try modinfo first, fall back to symlink parsing
fn get_module_version(module_path: &Path) -> Result<Option<(String, String)>> {
    if let Some((name, version)) = try_modinfo_version(module_path)? {
        return Ok(Some((name, version)));
    }
    parse_symlink_version(module_path)
}

fn try_modinfo_version(module_path: &Path) -> Result<Option<(String, String)>> {
    let ko_path = find_ko_file(module_path)?;
    let ko_path = match ko_path {
        Some(p) => p,
        None => return Ok(None),
    };

    let output = Command::new("modinfo")
        .arg(&ko_path)
        .output()
        .context("Failed to run modinfo")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut name = None;
    let mut version = None;

    for line in stdout.lines() {
        if let Some(n) = line.strip_prefix("name:") {
            name = Some(n.trim().to_string());
        }
        if let Some(v) = line.strip_prefix("version:") {
            let v = v.trim();
            if !v.starts_with('#') {
                version = Some(v.to_string());
            }
        }
    }

    match (name, version) {
        (Some(n), Some(v)) => Ok(Some((n, v))),
        _ => Ok(None),
    }
}

fn find_ko_file(dir: &Path) -> Result<Option<std::path::PathBuf>> {
    if !dir.is_dir() {
        return Ok(None);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains(".ko") {
            return Ok(Some(entry.path()));
        }
    }
    Ok(None)
}

// parse /nix/store/<hash>-<name>-<version>/...
fn parse_symlink_version(symlink_path: &Path) -> Result<Option<(String, String)>> {
    let target = fs::read_link(symlink_path).context("Failed to read symlink")?;
    let target_str = target.to_string_lossy();

    if let Some(store_part) = target_str.strip_prefix("/nix/store/") {
        if let Some(pkg_dir) = store_part.split('/').next() {
            if pkg_dir.len() > 33 {
                let name_version = &pkg_dir[33..]; // skip hash
                if let Some((name, version)) = split_name_version(name_version) {
                    return Ok(Some((name.to_string(), version.to_string())));
                }
            }
        }
    }

    Ok(None)
}

// split "foo-1.2.3" into ("foo", "1.2.3") at first digit
fn split_name_version(s: &str) -> Option<(&str, &str)> {
    let chars: Vec<char> = s.chars().collect();
    for i in 0..chars.len().saturating_sub(1) {
        if chars[i] == '-' && chars[i + 1].is_ascii_digit() {
            return Some((&s[..i], &s[i + 1..]));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_name_version_simple() {
        assert_eq!(split_name_version("xone-0.5.0"), Some(("xone", "0.5.0")));
    }

    #[test]
    fn test_split_name_version_unstable() {
        assert_eq!(
            split_name_version("xpad-noone-0-unstable-2024-01-10"),
            Some(("xpad-noone", "0-unstable-2024-01-10"))
        );
    }

    #[test]
    fn test_split_name_version_no_version() {
        assert_eq!(split_name_version("some-package-name"), None);
    }

    #[test]
    fn test_parse_modinfo_output() {
        // Simulating what we'd get from modinfo for nvidia
        let output = "filename:       /some/path/nvidia.ko\nversion:        590.48.01\nlicense:        NVIDIA\n";
        for line in output.lines() {
            if let Some(version) = line.strip_prefix("version:") {
                let version = version.trim();
                assert_eq!(version, "590.48.01");
                return;
            }
        }
        panic!("version not found");
    }

    #[test]
    fn test_parse_modinfo_placeholder_version() {
        // xone has #VERSION# placeholder
        let output = "version:        #VERSION#\n";
        for line in output.lines() {
            if let Some(version) = line.strip_prefix("version:") {
                let version = version.trim();
                assert!(version.starts_with('#'), "should detect placeholder");
                return;
            }
        }
    }
}
