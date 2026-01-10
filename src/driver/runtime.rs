use std::path::Path;

use serde::Serialize;

use crate::{
    Cli,
    config::{
        Manifest, ToolConfig,
        tool_config::{ArchiverKind, CCompilerKind, CppCompilerKind, LinkerKind},
    },
    driver::toolchain_resolve::resolve_toolchain,
    error::{CoreError, CoreResult},
    manifest::LibType,
    utils::get_number_of_max_parallel_threads,
};

#[derive(Debug, Clone, Serialize)]
pub struct RunTimeConfig<'a> {
    pub meta: Meta<'a>,
    pub profile: Profile,
    pub targets: Vec<Target<'a>>,
    pub toolchain: RuntimeToolchain,
}

#[derive(Debug, Clone, Serialize)]
pub struct Meta<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub threads: usize,
    pub edition: &'a str,
}

#[derive(Debug, Clone, Serialize)]
pub struct Profile {
    pub name: String,
    pub flags: Option<Vec<String>>,
    pub lto: bool,
    pub opt_level: i32,
    pub debug: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Target<'a> {
    pub name: &'a str,
    pub kind: TargetKind<'a>,
    pub flags: Option<Vec<String>>,
    pub ignore: &'a Option<Vec<String>>,
    pub defines: &'a Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub enum TargetKind<'a> {
    Static,
    Shared,
    Executable { entry: &'a Path },
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeToolchain {
    pub c_compiler: CCompilerKind,
    pub cpp_compiler: CppCompilerKind,
    pub linker: LinkerKind,
    pub archiver: ArchiverKind,
}

impl<'a> RunTimeConfig<'a> {
    /// Resolve the runtime configuration from the command line arguments, manifest contents,
    /// and tool config
    pub fn new(cli: &'a Cli, manifest: &'a Manifest, config: &'a ToolConfig) -> CoreResult<Self> {
        let meta = resolve_meta(cli, manifest, config)?;
        let profile = resolve_profile(cli, manifest, config)?;
        let targets = resolve_targets(cli, manifest, config)?;
        let toolchain = resolve_toolchain(config)?;

        Ok(Self {
            meta,
            profile,
            targets,
            toolchain,
        })
    }
}

fn resolve_meta<'a>(
    cli: &'a Cli,
    manifest: &'a Manifest,
    config: &'a ToolConfig,
) -> CoreResult<Meta<'a>> {
    let name = &manifest.package.name;
    let version = &manifest.package.version;
    let edition = &manifest.package.edition;
    let threads = cli
        .threads()
        .unwrap_or(
            config
                .build
                .as_ref()
                .and_then(|b| b.threads)
                .unwrap_or_else(get_number_of_max_parallel_threads),
        )
        .get();

    Ok(Meta {
        name,
        version,
        edition,
        threads,
    })
}

fn resolve_profile(cli: &Cli, manifest: &Manifest, config: &ToolConfig) -> CoreResult<Profile> {
    // NOTE: Use str instead of enum, to make custom profile extension easier later
    let name = cli.profile_name();
    let config_compiler_flags = config
        .build
        .as_ref()
        .and_then(|b| b.compiler_flags.as_ref());
    let profile = match name {
        "dev" => {
            let manifest_profile = manifest
                .profile
                .clone()
                .and_then(|profiles| profiles.dev)
                .unwrap_or(crate::manifest::Profile::default_dev());

            let flags = if let Some(config_flags) = config_compiler_flags
                && let Some(profile_flags) = &manifest_profile.flags
            {
                let mut flags = config_flags.clone();
                flags.extend_from_slice(profile_flags);
                Some(flags)
            } else {
                config_compiler_flags.cloned().or(manifest_profile.flags)
            };
            Profile {
                name: name.into(),
                flags,
                lto: manifest_profile.lto.unwrap_or(false),
                opt_level: manifest_profile.opt_level.unwrap_or(0),
                debug: manifest_profile.debug.unwrap_or(true),
            }
        }
        "release" => {
            let manifest_profile = manifest
                .profile
                .clone()
                .and_then(|profiles| profiles.release)
                .unwrap_or(crate::manifest::Profile::default_release());

            let flags = if let Some(config_flags) = config_compiler_flags
                && let Some(profile_flags) = &manifest_profile.flags
            {
                let mut flags = config_flags.clone();
                flags.extend_from_slice(profile_flags);
                Some(flags)
            } else {
                config_compiler_flags.cloned().or(manifest_profile.flags)
            };

            Profile {
                name: name.into(),
                flags,
                lto: manifest_profile.lto.unwrap_or(true),
                opt_level: manifest_profile.opt_level.unwrap_or(3),
                debug: manifest_profile.debug.unwrap_or(false),
            }
        }
        _ => return Err(Box::new(CoreError::UnresolvedProfileName(name.to_string()))),
    };
    Ok(profile)
}

fn resolve_targets<'a>(
    cli: &'a Cli,
    manifest: &'a Manifest,
    config: &'a ToolConfig,
) -> CoreResult<Vec<Target<'a>>> {
    let config_linker_flags = config.build.as_ref().and_then(|b| b.linker_flags.as_ref());

    let mut lib_target = None;
    if cli.bin().is_none()
        && let Some(lib) = &manifest.lib
    {
        let name = &lib.name;
        let kind = match lib.ty {
            LibType::Static => TargetKind::Static,
            LibType::Shared => TargetKind::Shared,
        };
        let ignore = &lib.ignore;

        let flags = if let Some(config_flags) = config_linker_flags
            && let Some(profile_flags) = &lib.flags
        {
            let mut flags = config_flags.clone();
            flags.extend_from_slice(profile_flags);
            Some(flags)
        } else {
            config_linker_flags.cloned().or(lib.flags.clone())
        };

        let defines = &lib.defines;
        lib_target = Some(Target {
            name,
            kind,
            ignore,
            flags,
            defines,
        });
    }

    let mut targets = vec![];
    if !cli.lib()
        && let Some(bins) = &manifest.bin
    {
        let cli_bin = cli.bin();
        for bin in bins {
            if let Some(name) = cli_bin
                && name != bin.name
            {
                continue;
            }

            if targets.iter().any(|t: &Target| t.name == bin.name) {
                return Err(Box::new(CoreError::ManifestValidation {
                    msg: format!("Multiple binaries named '{}' found", &bin.name),
                    help: "Consider removing or renaming binary target(s)".into(),
                }));
            }

            let name = &bin.name;
            let kind = TargetKind::Executable { entry: &bin.entry };
            let ignore = &bin.ignore;

            let flags = if let Some(config_flags) = config_linker_flags
                && let Some(profile_flags) = &bin.flags
            {
                let mut flags = config_flags.clone();
                flags.extend_from_slice(profile_flags);
                Some(flags)
            } else {
                config_linker_flags.cloned().or(bin.flags.clone())
            };

            let defines = &bin.defines;
            let bin_target = Target {
                name,
                kind,
                ignore,
                flags,
                defines,
            };
            targets.push(bin_target);
        }
    }

    if let Some(lib) = lib_target {
        targets.push(lib);
    }
    if targets.is_empty() {
        return Err(Box::new(CoreError::NoEligibleTarget));
    }
    Ok(targets)
}
