use avena::messages::WorkloadSpec;
use color_eyre::Result;
use tokio::fs;
use std::path::Path;

pub struct WorkloadDeployment {
    pub name: String,
    pub spec: WorkloadSpec,
}

impl WorkloadDeployment {
    pub async fn deploy(&self, systemd_dir: &Path) -> Result<()> {
        fs::create_dir_all(systemd_dir).await?;

        let mut quadlet = format!(
            "[Unit]\nDescription={}\n\n[Container]\nContainerName={}\nImage={}",
            self.name,
            self.name,
            if let Some(tag) = &self.spec.tag {
                format!("{}:{}", self.spec.image, tag)
            } else {
                self.spec.image.clone()
            }
        );

        if let Some(cmd) = &self.spec.cmd {
            quadlet.push_str(&format!("\nExec={}", cmd));
        }

        for port in &self.spec.ports {
            quadlet.push_str(&format!(
                "\nPublishPort={}:{}",
                port.host, port.container
            ));
        }

        for mount in &self.spec.mounts {
            let readonly_flag = if mount.readonly { "ro" } else { "z" };
            quadlet.push_str(&format!("\nVolume={}:{}:{}", mount.host, mount.container, readonly_flag));
        }

        for vol_name in &self.spec.volumes {
            quadlet.push_str(&format!("\nVolume={}.volume:/data", vol_name));
        }

        quadlet.push_str("\n\n[Service]\nRestart=on-failure\n");

        fs::write(systemd_dir.join(format!("{}.container", self.name)), quadlet).await?;

        for vol_name in &self.spec.volumes {
            fs::write(
                systemd_dir.join(format!("{}.volume", vol_name)),
                "[Volume]\n",
            ).await?;
        }

        Ok(())
    }
}
