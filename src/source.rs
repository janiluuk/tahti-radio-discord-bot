use crate::track::{Track, TrackMetadata};
use crate::ytdlp;

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error(transparent)]
    Ytdlp(#[from] ytdlp::YtdlpError),

    #[error("yt-dlp output was missing expected fields")]
    MissingFields,
}

pub async fn resolve(youtube_url: &str) -> Result<Track, SourceError> {
    let output = ytdlp::fetch_metadata_and_stream_url(youtube_url).await?;
    let mut lines = output.lines();

    let metadata = TrackMetadata::parse(youtube_url, &mut lines)
        .ok_or(SourceError::MissingFields)?;
    let stream_url = lines.next()
        .map(str::to_owned)
        .ok_or(SourceError::MissingFields)?;

    Ok(Track { metadata, stream_url })
}

pub async fn resolve_metadata(youtube_url: &str) -> Result<TrackMetadata, SourceError> {
    let output = ytdlp::fetch_metadata(youtube_url).await?;
    let mut lines = output.lines();

    TrackMetadata::parse(youtube_url, &mut lines)
        .ok_or(SourceError::MissingFields)
}
