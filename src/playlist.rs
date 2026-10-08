const TRACKS: &str = include_str!("../tracks.txt");

#[derive(Debug, thiserror::Error)]
pub enum PlaylistError {
    #[error("no tracks found in tracks.txt")]
    Empty,
}

/// Prefer `TAHTI_RADIO_AUDIO_URL` (or legacy `TAHTI_RADIO_HLS_URL`) when set —
/// that is the live Tahti Radio HLS/direct stream. Otherwise fall back to the
/// curated YouTube `tracks.txt` playlist.
///
/// Note: `GET /api/v1/radio` on the Tahti API is now-playing **metadata**, not
/// an audio URL. Do not point the bot at that route for playback.
pub fn load() -> Result<Vec<String>, PlaylistError> {
    if let Some(url) = env_stream_url() {
        return Ok(vec![url]);
    }

    let tracks: Vec<String> = TRACKS
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect();

    if tracks.is_empty() {
        return Err(PlaylistError::Empty);
    }

    Ok(tracks)
}

fn env_stream_url() -> Option<String> {
    for key in ["TAHTI_RADIO_AUDIO_URL", "TAHTI_RADIO_HLS_URL"] {
        if let Ok(value) = std::env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_owned());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_returns_non_empty_from_tracks_txt() {
        // Env may be set in CI — clear for this unit test.
        std::env::remove_var("TAHTI_RADIO_AUDIO_URL");
        std::env::remove_var("TAHTI_RADIO_HLS_URL");
        let tracks = load().expect("tracks.txt should load");
        assert!(!tracks.is_empty());
        assert!(tracks.iter().all(|u| u.starts_with("http")));
    }

    #[test]
    fn load_prefers_tahti_radio_audio_url() {
        std::env::set_var(
            "TAHTI_RADIO_AUDIO_URL",
            "https://stream.example/hls/tahti-radio/master.m3u8",
        );
        let tracks = load().expect("env playlist");
        assert_eq!(
            tracks,
            vec!["https://stream.example/hls/tahti-radio/master.m3u8".to_owned()]
        );
        std::env::remove_var("TAHTI_RADIO_AUDIO_URL");
    }
}
