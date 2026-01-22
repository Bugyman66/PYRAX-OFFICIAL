use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::config::InfernoConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstanceStatus {
    Running,
    Stopped,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub instance_id: u32,
    pub status: InstanceStatus,
    pub pid: Option<u32>,
    pub config: Option<InfernoConfig>,
    pub uptime: Option<String>,
}

pub struct InstanceManager {
    base_dir: PathBuf,
}

impl InstanceManager {
    pub fn new() -> Result<Self> {
        let base_dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(".inferno");

        fs::create_dir_all(&base_dir)?;

        Ok(Self { base_dir })
    }

    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }

    pub fn instance_dir(&self, instance_id: u32) -> PathBuf {
        self.base_dir
            .join("devnet")
            .join(format!("instance-{}", instance_id))
    }

    pub fn config_path(&self, instance_id: u32) -> PathBuf {
        self.instance_dir(instance_id).join("config.toml")
    }

    pub fn pid_path(&self, instance_id: u32) -> PathBuf {
        self.instance_dir(instance_id).join("inferno.pid")
    }

    pub fn instance_exists(&self, instance_id: u32) -> bool {
        self.config_path(instance_id).exists()
    }

    pub fn next_available_instance(&self) -> u32 {
        for i in 1..=100 {
            if !self.instance_exists(i) {
                return i;
            }
        }
        1
    }

    pub fn list_instances(&self) -> Result<Vec<InstanceInfo>> {
        let mut instances = Vec::new();

        let devnet_dir = self.base_dir.join("devnet");
        if !devnet_dir.exists() {
            return Ok(instances);
        }

        for entry in fs::read_dir(&devnet_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if let Some(id_str) = name.strip_prefix("instance-") {
                    if let Ok(instance_id) = id_str.parse::<u32>() {
                        let config = self.load_config(instance_id).ok();
                        let pid = self.get_pid(instance_id).ok().flatten();
                        let is_running = pid.map(|p| self.is_process_running(p)).unwrap_or(false);

                        let status = if is_running {
                            InstanceStatus::Running
                        } else {
                            InstanceStatus::Stopped
                        };

                        let uptime = if is_running {
                            pid.and_then(|p| self.get_process_uptime(p))
                        } else {
                            None
                        };

                        instances.push(InstanceInfo {
                            instance_id,
                            status,
                            pid,
                            config,
                            uptime,
                        });
                    }
                }
            }
        }

        instances.sort_by_key(|i| i.instance_id);
        Ok(instances)
    }

    pub fn load_config(&self, instance_id: u32) -> Result<InfernoConfig> {
        let config_path = self.config_path(instance_id);
        let content = fs::read_to_string(&config_path)
            .context(format!("Failed to read config file: {}", config_path.display()))?;
        let config: InfernoConfig = toml::from_str(&content)
            .context("Failed to parse config file")?;
        Ok(config)
    }

    pub fn save_config(&self, instance_id: u32, config: &InfernoConfig) -> Result<()> {
        let instance_dir = self.instance_dir(instance_id);
        fs::create_dir_all(&instance_dir)?;

        // Also create data directory
        fs::create_dir_all(&config.node.data_dir)?;

        let config_path = self.config_path(instance_id);
        let content = toml::to_string_pretty(config)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn get_pid(&self, instance_id: u32) -> Result<Option<u32>> {
        let pid_path = self.pid_path(instance_id);
        if !pid_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&pid_path)?;
        let pid: u32 = content.trim().parse()?;
        Ok(Some(pid))
    }

    pub fn save_pid(&self, instance_id: u32, pid: u32) -> Result<()> {
        let pid_path = self.pid_path(instance_id);
        fs::write(&pid_path, pid.to_string())?;
        Ok(())
    }

    pub fn remove_pid(&self, instance_id: u32) -> Result<()> {
        let pid_path = self.pid_path(instance_id);
        if pid_path.exists() {
            fs::remove_file(&pid_path)?;
        }
        Ok(())
    }

    pub fn is_process_running(&self, pid: u32) -> bool {
        #[cfg(unix)]
        {
            use std::process::Command;
            Command::new("kill")
                .args(["-0", &pid.to_string()])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }

        #[cfg(windows)]
        {
            use sysinfo::{Pid, System};
            let mut system = System::new();
            system.refresh_processes();
            system.process(Pid::from(pid as usize)).is_some()
        }
    }

    pub fn kill_process(&self, pid: u32, force: bool) -> Result<()> {
        #[cfg(unix)]
        {
            use std::process::Command;
            let signal = if force { "-9" } else { "-15" };
            Command::new("kill")
                .args([signal, &pid.to_string()])
                .output()
                .context("Failed to kill process")?;

            if !force {
                // Wait for graceful shutdown
                for _ in 0..30 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    if !self.is_process_running(pid) {
                        return Ok(());
                    }
                }
                // Force kill if still running
                Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .output()?;
            }
        }

        #[cfg(windows)]
        {
            use std::process::Command;
            let pid_str = pid.to_string();
            let args: Vec<&str> = if force {
                vec!["/F", "/PID", &pid_str]
            } else {
                vec!["/PID", &pid_str]
            };
            Command::new("taskkill")
                .args(args)
                .output()
                .context("Failed to kill process")?;
        }

        Ok(())
    }

    pub fn get_process_uptime(&self, _pid: u32) -> Option<String> {
        // Platform-specific implementation
        // For now, return None
        None
    }
}

impl Clone for InstanceManager {
    fn clone(&self) -> Self {
        Self {
            base_dir: self.base_dir.clone(),
        }
    }
}
