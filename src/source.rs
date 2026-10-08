use std::time::Duration;

use crate::track::{Track, TrackMetadata};
use crate::ytdlp;

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error(transparent)]
    Ytdlp(#[from] ytdlp::YtdlpError),

    #[error("yt-dlp output was missing expected fields")]
    MissingFields,
}

/// HLS playlists and other direct audio URLs skip yt-dlp; ffmpeg can open them.
pub fn is_direct_stream_url(url: &str) -> bool {
    let lower = url.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return false;
    }
    if lower.contains(".m3u8") || lower.contains(".m3u?") || lower.ends_with(".m3u") {
        return true;
    }
    if lower.ends_with(".mp3")
        || lower.ends_with(".aac")
        || lower.ends_with(".ogg")
        || lower.ends_with(".opus")
        || lower.ends_with(".flac")
    {
        return true;
    }
    // Icecast-style mounts often have no extension; treat non-YouTube http(s) as direct
    // when the host looks like a stream CDN. Keep YouTube on the yt-dlp path.
    if (lower.starts_with("http://") || lower.starts_with("https://"))
        && !lower.contains("youtube.com")
        && !lower.contains("youtu.be")
    {
        // Only auto-direct when the path suggests HLS/stream, or env-style radio URL.
        return lower.contains("/hls/")
            || lower.contains("icecast")
            || lower.contains("/live/")
            || lower.contains("master.m3u8")
            || lower.contains("index.m3u8");
    }
    false
}

fn direct_track(url: &str) -> Track {
    Track {
        metadata: TrackMetadata {
            artist: Some("Tahti Radio".to_owned()),
            title: Some("Live".to_owned()),
            video_title: "Tahti Radio".to_owned(),
            duration: Duration::ZERO,
            youtube_url: url.to_owned(),
            thumbnail_url: String::new(),
        },
        stream_url: url.to_owned(),
    }
}

pub async fn resolve(url: &str) -> Result<Track, SourceError> {
    if is_direct_stream_url(url) {
        return Ok(direct_track(url));
    }

    let output = ytdlp::fetch_metadata_and_stream_url(url).await?;
    let mut lines = output.lines();

    let metadata = TrackMetadata::parse(url, &mut lines).ok_or(SourceError::MissingFields)?;
    let stream_url = lines
        .next()
        .map(str::to_owned)
        .ok_or(SourceError::MissingFields)?;

    Ok(Track {
        metadata,
        stream_url,
    })
}

pub async fn resolve_metadata(url: &str) -> Result<TrackMetadata, SourceError> {
    if is_direct_stream_url(url) {
        return Ok(direct_track(url).metadata);
    }

    let output = ytdlp::fetch_metadata(url).await?;
    let mut lines = output.lines();

    TrackMetadata::parse(url, &mut lines).ok_or(SourceError::MissingFields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_hls_urls() {
        assert!(is_direct_stream_url(
            "https://cdn.example/hls/tahti-radio/master.m3u8"
        ));
        assert!(is_direct_stream_url(
            "https://cdn.example/tahti-radio/index.m3u8?token=1"
        ));
        assert!(!is_direct_stream_url(
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
        ));
    }

    #[tokio::test]
    async fn resolve_direct_skips_ytdlp() {
        let track = resolve("https://stream.example/hls/live/master.m3u8")
            .await
            .expect("direct");
        assert_eq!(track.stream_url, "https://stream.example/hls/live/master.m3u8");
        assert_eq!(track.metadata.artist.as_deref(), Some("Tahti Radio"));
    }
}
