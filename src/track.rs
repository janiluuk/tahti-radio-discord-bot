use std::fmt;
use std::str::Lines;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct TrackMetadata {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub video_title: String,
    pub duration: Duration,
    pub youtube_url: String,
    pub thumbnail_url: String,
}

impl TrackMetadata {
    pub fn parse(youtube_url: &str, lines: &mut Lines<'_>) -> Option<Self> {
        let video_title = required_field(lines)?;
        let artist = optional_field(lines);
        let title = optional_field(lines);
        let duration_secs = required_field(lines)?
            .parse::<u64>()
            .unwrap_or(0);
        let thumbnail_url = required_field(lines)?;

        Some(Self {
            artist,
            title,
            video_title,
            duration: Duration::from_secs(duration_secs),
            youtube_url: youtube_url.to_owned(),
            thumbnail_url,
        })
    }
}

impl fmt::Display for TrackMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.artist, &self.title) {
            (Some(artist), Some(title)) => write!(f, "{artist} - {title}"),
            _ => write!(f, "{}", self.video_title),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Track {
    pub metadata: TrackMetadata,
    pub stream_url: String,
}

impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.metadata.fmt(f)
    }
}

fn required_field(lines: &mut Lines<'_>) -> Option<String> {
    lines
        .next()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
}

fn optional_field(lines: &mut Lines<'_>) -> Option<String> {
    lines
        .next()
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "NA")
        .map(str::to_owned)
}
