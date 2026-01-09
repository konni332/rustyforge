use std::{collections::HashMap, path::Path};

use crate::{
    CoreResult,
    config::tool_config::{Build, ToolConfig, Toolchain},
};

impl ToolConfig {
    pub fn resolve_config() -> CoreResult<Self> {
        let global = Self::read_global()?.unwrap_or_default();
        let local_opt = Self::read_local()?;
        if let Some(local) = local_opt {
            Ok(Self::merge_configs(global, local))
        } else {
            Ok(global)
        }
    }
    fn read_global() -> CoreResult<Option<Self>> {
        let global_config_dir = match dirs_next::config_dir() {
            Some(path) => path,
            None => return Ok(None),
        };
        let path = global_config_dir.join(".rustyforge").join("config.toml");
        if path.exists() {
            let config = Self::read_config(&path)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    fn read_local() -> CoreResult<Option<Self>> {
        let cwd = std::env::current_dir()?;
        let path = cwd.join(".rustyforge").join("config.toml");
        if path.exists() {
            let config = Self::read_config(&path)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    fn read_config(path: &Path) -> CoreResult<Self> {
        let src = std::fs::read_to_string(path)?;
        let config: ToolConfig = toml::from_str(&src).map_err(|e| {
            Box::new(crate::CoreError::InvalidConfig {
                path: path.to_path_buf(),
                src,
                span: e.span().unwrap_or_default(),
                msg: e.message().to_string(),
            })
        })?;
        Ok(config)
    }

    fn merge_configs(global: ToolConfig, local: ToolConfig) -> Self {
        let build = if global.build.is_some() && local.build.is_some() {
            Some(Self::merge_build(
                global.build.unwrap(),
                local.build.unwrap(),
            ))
        } else {
            local.build.or(global.build)
        };

        let toolchain = if global.toolchain.is_some() && local.toolchain.is_some() {
            Some(Self::merge_toolchains(
                global.toolchain.unwrap(),
                local.toolchain.unwrap(),
            ))
        } else {
            local.toolchain.or(global.toolchain)
        };

        Self { build, toolchain }
    }
    fn merge_toolchains(
        global: HashMap<String, Toolchain>,
        mut local: HashMap<String, Toolchain>,
    ) -> HashMap<String, Toolchain> {
        for (key, value) in global {
            local.entry(key).or_insert(value);
        }
        local
    }
    fn merge_build(global: Build, local: Build) -> Build {
        let threads = local.threads.or(global.threads);
        let compiler_flags = local.compiler_flags.or(global.compiler_flags);
        let linker_flags = local.linker_flags.or(global.linker_flags);
        Build {
            threads,
            compiler_flags,
            linker_flags,
        }
    }
}
