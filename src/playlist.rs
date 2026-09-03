const TRACKS: &str = include_str!("../tracks.txt");

#[derive(Debug, thiserror::Error)]
pub enum PlaylistError {
    #[error("no tracks found in tracks.txt")]
    Empty,
}

pub fn load() -> Result<Vec<String>, PlaylistError> {
    let tracks: Vec<String> = TRACKS.lines().map(str::to_owned).collect();

    if tracks.is_empty() {
        return Err(PlaylistError::Empty);
    }

    Ok(tracks)
}
