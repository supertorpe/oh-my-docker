use bollard::Docker;
use anyhow::Result;

pub fn connect() -> Result<Docker> {
    let docker = Docker::connect_with_local_defaults()?;
    Ok(docker)
}
