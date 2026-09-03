use std::process::Stdio;

use tokio::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum YtdlpError {
    #[error("failed to run yt-dlp: {0}")]
    Spawn(#[from] std::io::Error),

    #[error("yt-dlp exited unsuccessfully: {0}")]
    Failed(String),
}

pub async fn fetch_metadata(url: &str) -> Result<String, YtdlpError> {
    run(url, false).await
}

pub async fn fetch_metadata_and_stream_url(url: &str) -> Result<String, YtdlpError> {
    run(url, true).await
}

async fn run(url: &str, include_stream_url: bool) -> Result<String, YtdlpError> {
    let mut cmd = Command::new("yt-dlp");

    cmd.args([
        "-f", "bestaudio",
        "--print", "%(title)s",
        "--print", "%(artist)s",
        "--print", "%(track)s",
        "--print", "%(duration)s",
        "--print", "%(thumbnail)s",
    ]);

    if include_stream_url {
        cmd.arg("-g");
    }

    cmd.arg(url);

    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(YtdlpError::Failed(stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
