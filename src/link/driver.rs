use std::{path::PathBuf, sync::Arc, time::Duration};

use anyhow::{Result, bail};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;

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

pub struct LinkerDirver<'a, L: Linker + Sync> {
    pub linker: L,
    pub opts: LinkOptions,
    pub project_config: &'a ProjectConfig,
    pub objects: Vec<PathBuf>,
    pub profile: Profile,
}

impl<'a, L: Linker + Sync> LinkerDirver<'a, L> {
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
    pub fn link_single_target(
        &self,
        link_target: &LinkTarget,
        pb: &ProgressBar,
    ) -> anyhow::Result<PathBuf> {
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
                let msg = ui::error_link_msg(None, format!("{}", e).as_bytes(), link_target.kind);
                pb.println(msg);
                bail!("failed to spawn link command");
            }
        };
        let mut cmd = std::process::Command::from(&cannonical);

        let output = match cmd.output() {
            Ok(output) => output,
            Err(e) => {
                let msg =
                    ui::error_link_msg(Some(&cmd), format!("{}", e).as_bytes(), link_target.kind);
                pb.println(msg);
                bail!("failed to spawn link command");
            }
        };

        if output.status.success() {
            let msg = ui::successfull_link_msg(&cmd, link_target.kind);
            pb.println(msg);
        } else {
            let msg = ui::error_link_msg(Some(&cmd), &output.stderr, link_target.kind);
            pb.println(msg);
            bail!("failed to link");
        }

        Ok(output_path)
    }
    pub fn link(&self) -> anyhow::Result<Option<PathBuf>> {
        let targets = if self
            .project_config
            .build
            .link_target
            .as_ref()
            .is_some_and(|lt| !lt.is_empty())
        {
            self.project_config.build.link_target.clone().unwrap()
        } else {
            vec![LinkTarget {
                name: self.project_config.project.name.clone(),
                kind: LinkTargetKind::Executable,
                user_flags: None,
            }]
        };

        let mp = Arc::new(MultiProgress::new());

        let total_pb = mp.add(ProgressBar::new(targets.len() as u64));
        total_pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40}] {pos}/{len} ({eta}) {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );

        let results: Vec<_> = targets
            .into_par_iter()
            .map(|link_target| {
                // Spinner für diesen Target
                let spinner = mp.add(ProgressBar::new_spinner());
                spinner.set_message(format!("Linking {}", link_target.name));
                spinner.enable_steady_tick(Duration::from_millis(100));

                // Link-Vorgang
                let res = self.link_single_target(&link_target, &spinner);

                spinner.finish_and_clear();

                // Gesamt-PB inkrementieren
                total_pb.inc(1);

                (link_target, res)
            })
            .collect();

        total_pb.finish_and_clear();
        // Fehlerbehandlung
        let mut failed = false;
        let mut executable_path = None;
        for (link_target, res) in results {
            if res.is_err() {
                failed = true;
            } else if let Ok(path) = res
                && link_target.kind == LinkTargetKind::Executable
            {
                executable_path = Some(path);
            }
        }

        if failed {
            bail!("at least one of the linking targets failed");
        } else {
            println!("{}", ui::finished_linking_msg(total_pb.elapsed()));
        }
        Ok(executable_path)
    }
}
