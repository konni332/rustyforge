use std::{
    io::{Write, stderr},
    path::PathBuf,
};

use anyhow::{Result, bail};

use crate::{
    ForgeArgs,
    compile::types::Profile,
    config::project::{LinkTarget, LinkTargetKind, ProjectConfig},
    fs::{object_dir, profile_dir},
    link::{
        Linker,
        types::{LinkOptions, LinkUnit},
    },
    ui::{self},
};

pub struct LinkerDirver<'a, L: Linker> {
    pub linker: L,
    pub opts: LinkOptions,
    pub project_config: &'a ProjectConfig,
    pub objects: Vec<PathBuf>,
    pub profile: Profile,
}

impl<'a, L: Linker> LinkerDirver<'a, L> {
    pub fn default_driver(
        args: &ForgeArgs,
        project_config: &'a ProjectConfig,
    ) -> anyhow::Result<Self> {
        let profile = args.profile().unwrap_or(Profile::Debug);

        let target = args.target();

        let opts = LinkOptions {
            target: target.cloned(),
        };
        let linker = L::new();
        let objects = Self::discover_objects(profile)?;
        Ok(Self {
            profile,
            linker,
            objects,
            opts,
            project_config,
        })
    }
    pub fn discover_objects(profile: Profile) -> Result<Vec<PathBuf>> {
        let obj_dir = object_dir(profile_dir(profile)?);
        Ok(jwalk::WalkDir::new(obj_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|e| e.to_str()) == Some("o"))
            .map(|e| e.path().to_path_buf())
            .collect())
    }
    pub fn resolve_output_path(
        &self,
        profile: Profile,
        link_target: &LinkTarget,
    ) -> Result<PathBuf> {
        let mut profile_dir = profile_dir(profile)?;
        match link_target.kind {
            LinkTargetKind::Executable => {
                #[cfg(target_os = "windows")]
                profile_dir.push(&format!("{}.exe", &link_target.name));

                #[cfg(any(target_os = "linux", target_os = "macos"))]
                profile_dir.push(&link_target.name);

                #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
                compile_error!("Unsupported OS");
            }
            LinkTargetKind::StaticLibrary => {
                #[cfg(target_os = "windows")]
                profile_dir.push(&format!("{}.lib", &link_target.name));

                #[cfg(any(target_os = "linux", target_os = "macos"))]
                profile_dir.push(format!("{}.a", link_target.name));

                #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
                compile_error!("Unsupported OS");
            }
            LinkTargetKind::SharedLibrary => {
                #[cfg(target_os = "windows")]
                profile_dir.push(&format!("{}.dll", &link_target.name));

                #[cfg(target_os = "linux")]
                profile_dir.push(format!("{}.so", link_target.name));

                #[cfg(target_os = "macos")]
                profile_dir.push(&format!("{}.dylib", &link_target.name));

                #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
                compile_error!("Unsupported OS");
            }
        }
        Ok(profile_dir)
    }
    pub fn link_single_target(&self, link_target: &LinkTarget) -> anyhow::Result<PathBuf> {
        let mut lib_dirs = vec![];
        let mut libs = vec![];

        let output_path = self.resolve_output_path(self.profile, link_target)?;

        for dep in &self.project_config.dependencies {
            lib_dirs.extend_from_slice(&dep.lib_dirs);
            libs.extend_from_slice(&dep.libs);
        }

        let user_flags = &link_target.user_flags.clone().unwrap_or(vec![]);
        let unit = LinkUnit {
            user_flags,
            kind: link_target.kind,
            objects: &self.objects,
            output: output_path.clone(),
            lib_dirs,
            libs,
        };
        let cannonical = match self.linker.link_cmd(&unit, &self.opts) {
            Ok(c) => c,
            Err(e) => {
                ui::output_error_link(None, format!("{}", e).as_bytes(), link_target.kind);
                stderr().flush()?;
                bail!("failed to spawn link command");
            }
        };
        let mut cmd = std::process::Command::from(&cannonical);

        let output = match cmd.output() {
            Ok(output) => output,
            Err(e) => {
                ui::output_error_link(Some(&cmd), format!("{}", e).as_bytes(), link_target.kind);
                stderr().flush()?;
                bail!("failed to spawn link command");
            }
        };
        if output.status.success() {
            ui::output_successfull_link(&cmd, link_target.kind);
        } else {
            ui::output_error_link(Some(&cmd), &output.stderr, link_target.kind);
            bail!("failed to link");
        }

        Ok(output_path)
    }
    pub fn link(&self) -> anyhow::Result<Option<PathBuf>> {
        let mut failed = false;
        let mut executable_path = None;
        let targets = if !self
            .project_config
            .build
            .link_targets
            .as_ref()
            .is_none_or(|lt| lt.is_empty())
        {
            self.project_config.build.link_targets.clone().unwrap()
        } else {
            vec![LinkTarget {
                name: self.project_config.project.name.clone(),
                kind: LinkTargetKind::Executable,
                user_flags: None,
            }]
        };
        for link_target in targets {
            let res = self.link_single_target(&link_target);
            if res.is_err() {
                failed = true;
            } else if link_target.kind == LinkTargetKind::Executable {
                executable_path = Some(res?);
            }
        }

        if failed {
            stderr().flush()?;
            bail!("at least one of the linking targets failed");
        }
        Ok(executable_path)
    }
}
