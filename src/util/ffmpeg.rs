use std::process::Command;

use anyhow::{bail, Result};

pub fn is_ffmpeg_installed() -> Result<()> {
    let ffmpeg_output = Command::new("ffmpeg").arg("-version").output()?;

    let ffprobe_output = Command::new("ffprobe").arg("-version").output()?;

    match (
        ffmpeg_output.status.success(),
        ffprobe_output.status.success(),
    ) {
        (true, true) => Ok(()),
        (false, true) => bail!(
            "ffmpeg is not installed or not in PATH: {}",
            String::from_utf8_lossy(&ffmpeg_output.stderr)
        ),
        (true, false) => bail!(
            "ffprobe is not installed or not in PATH: {}",
            String::from_utf8_lossy(&ffprobe_output.stderr)
        ),
        (false, false) => bail!(
            "ffmpeg and ffprobe are not installed or not in PATH: {}",
            String::from_utf8_lossy(&ffmpeg_output.stderr)
        ),
    }
}
