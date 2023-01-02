use std::collections::BTreeMap;
use std::env;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::Deserialize;

use crate::apply_colors::apply_colors;
use crate::scheme::ColorScheme;
use crate::utils::{
    create_dir_recursive, determine_home, read_into_buffer, write_buffer, Context, Result,
};

#[derive(Debug)]
pub struct AppConfig {
    pub scheme: String,
}

#[derive(Debug)]
pub struct App {
    scheme: ColorScheme,
    template_dir: PathBuf,
    targets: TargetList,
    workdir: PathBuf,
    shell: String,
}

impl App {
    pub fn init_config_dir() -> Result<()> {
        let config_dir = Self::determine_config_dir()?;

        println!("Initializing {}", config_dir.display());

        for name in ["templates", "schemes"] {
            create_dir_recursive(config_dir.join(name))?;
        }

        let targets_toml = config_dir.join("targets.toml");
        OpenOptions::new()
            .create_new(true)
            .read(false)
            .write(true)
            .open(&targets_toml)
            .add_context(|| format!("Failed to create {}", targets_toml.display()))?;

        Ok(())
    }

    pub fn try_from_config(config: AppConfig) -> Result<App> {
        let config_dir = Self::determine_config_dir()?;

        let scheme =
            ColorScheme::try_from_yaml(config_dir.join("schemes").join(config.scheme + ".yaml"))?;

        let targets = targets_from_toml(&config_dir.join("targets.toml"))?;

        if targets.len() == 0 {
            println!("Warning: No targets found in {}", config_dir.display());
        }

        Ok(App {
            targets,
            scheme,
            template_dir: config_dir.join("templates"),
            workdir: config_dir,
            shell: env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_owned()),
        })
    }

    pub fn run(self) -> Result<()> {
        for (name, config) in self.targets.iter() {
            if let Some(cmd) = &config.enable_if {
                if !self.precondition_satisfied(cmd)? {
                    continue;
                }
            };

            self.process_target(&name, &config)?;
        }

        Ok(())
    }

    fn process_target(&self, name: &TargetName, config: &TargetConfig) -> Result<()> {
        let target_file = Self::expand_path(&config.rewrite)?;

        if !target_file.is_absolute() {
            return Err(format!("Expected absolute path, found {}", target_file.display()).into());
        }

        // If the parent directory does not exist, create it if requested or return without
        // error.
        if let Some(dirname) = target_file.parent() {
            if !dirname.exists() {
                if config.mkdir {
                    create_dir_recursive(dirname)?;
                } else {
                    return Ok(());
                }
            }
        }

        let template_path = self
            .template_dir
            .join(config.template.as_ref().unwrap_or(name).to_owned() + ".mustache");

        let input_buffer = read_into_buffer(&template_path)?;

        let output_buffer = apply_colors(&self.scheme, &input_buffer)
            .add_context(|| format!("While processing {}", template_path.display()))?;

        write_buffer(&output_buffer, &target_file)?;

        if let Some(hook) = &config.hook {
            self.run_hook(hook)?;
        }

        Ok(())
    }

    fn expand_path(path: impl AsRef<Path>) -> Result<PathBuf> {
        if let Ok(stripped) = path.as_ref().strip_prefix("~") {
            Ok(determine_home()?.join(stripped))
        } else {
            Ok(path.as_ref().into())
        }
    }

    fn run_hook(&self, hook: &str) -> Result<()> {
        let mut cmd = Command::new(&self.shell);

        cmd.arg("-c").arg(hook);

        // Setup environment

        cmd.current_dir(&self.workdir)
            .env("SCHEME_AUTHOR", &self.scheme.author)
            .env("SCHEME_NAME", &self.scheme.name);

        for n in 0..16 {
            cmd.env(format!("BASE{:02X}", n), self.scheme.colors[n].to_string());
        }

        let exit_status = cmd
            .status()
            .add_context(|| format!("While running hook `{}`", hook))?;

        match exit_status.code() {
            Some(code) if code != 0 => {
                println!("Hook `{}` finished with exit status {}", hook, code)
            }
            None => println!("Hook `{}` was terminated by signal", hook),
            _ => {}
        }

        Ok(())
    }

    fn precondition_satisfied(&self, cmd: &str) -> Result<bool> {
        let status = Command::new(&self.shell)
            .arg("-c")
            .arg(cmd)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .add_context(|| format!("Failed to execute '{}'", cmd))?;

        Ok(status.success())
    }

    fn determine_config_dir() -> Result<PathBuf> {
        env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|_| determine_home().map(|dir| dir.join(".config")))
            .map(|dir| dir.join("base16-colorizer"))
            .add_context(|| "Failed to determine config directory")
    }
}

#[derive(Deserialize, Debug)]
struct TargetConfig {
    rewrite: PathBuf,
    #[serde(default)]
    template: Option<String>,
    #[serde(default)]
    enable_if: Option<String>,
    #[serde(default)]
    hook: Option<String>,
    #[serde(default)]
    mkdir: bool,
}

type TargetName = String;
type TargetList = BTreeMap<TargetName, TargetConfig>;

fn targets_from_toml(path: impl AsRef<Path>) -> Result<TargetList> {
    // The `toml` crate only provides a `from_str` function, so we read the whole file
    // into a buffer first.
    Ok(toml::from_str(&read_into_buffer(&path)?)
        .add_context(|| format!("In {}", path.as_ref().display()))?)
}
