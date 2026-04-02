//! Media framework via tarang
//!
//! Bridges tarang's media stack with kiran's ECS:
//! - **tarang** — Container parsing, audio/video codecs, media analysis, fingerprinting
//!
//! Core types:
//! - [`MediaSource`] component for entities that play media (cutscenes, in-game screens)
//! - Re-exports for codecs, containers, buffers, and AI analysis

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// tarang — media framework
// ---------------------------------------------------------------------------

/// Core media types (codecs, formats, buffers, metadata).
pub use tarang::core;

/// Audio decoding, encoding, resampling, mixing.
pub use tarang::audio as media_audio;

/// Container demuxing and muxing (MP4, MKV, WebM, Ogg, WAV).
pub use tarang::demux;

/// Video decoding, encoding, pixel format conversion, scaling.
pub use tarang::video;

/// Media analysis, fingerprinting, scene detection, transcription.
pub use tarang::ai as media_ai;

pub use tarang::core::TarangError;

// ---------------------------------------------------------------------------
// Media source component
// ---------------------------------------------------------------------------

/// Media source attached to an entity for playback (cutscenes, in-game video, music).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaSource {
    /// Path or URI to the media file.
    pub source: String,
    /// Current playback state.
    pub state: PlaybackState,
    /// Playback volume (0.0–1.0).
    pub volume: f32,
    /// Whether playback loops.
    pub looping: bool,
    /// Current playback position in seconds.
    #[serde(skip)]
    pub position_secs: f64,
    /// Total duration in seconds (populated after probe).
    #[serde(skip)]
    pub duration_secs: f64,
}

/// Playback state for a media source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum PlaybackState {
    /// Not yet started or reset.
    #[default]
    Idle,
    /// Currently playing.
    Playing,
    /// Paused.
    Paused,
    /// Playback finished.
    Finished,
    /// Error occurred during playback.
    Error,
}

impl MediaSource {
    /// Create a new media source from a file path.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "media")] {
    /// use kiran::media::MediaSource;
    ///
    /// let src = MediaSource::new("cutscenes/intro.mp4");
    /// assert_eq!(src.state, kiran::media::PlaybackState::Idle);
    /// # }
    /// ```
    pub fn new(source: impl Into<String>) -> Self {
        let source = source.into();
        tracing::trace!(%source, "created media source");
        Self {
            source,
            state: PlaybackState::Idle,
            volume: 1.0,
            looping: false,
            position_secs: 0.0,
            duration_secs: 0.0,
        }
    }

    /// Set playback volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Enable looping.
    pub fn with_looping(mut self) -> Self {
        self.looping = true;
        self
    }

    /// Whether playback is active (playing, not paused/finished/error).
    #[must_use]
    #[inline]
    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_source_builder() {
        let src = MediaSource::new("video/intro.mp4")
            .with_volume(0.8)
            .with_looping();
        assert_eq!(src.source, "video/intro.mp4");
        assert_eq!(src.volume, 0.8);
        assert!(src.looping);
        assert_eq!(src.state, PlaybackState::Idle);
        assert!(!src.is_playing());
    }

    #[test]
    fn media_source_default_state() {
        let src = MediaSource::new("music/theme.ogg");
        assert_eq!(src.state, PlaybackState::Idle);
        assert_eq!(src.volume, 1.0);
        assert!(!src.looping);
    }

    #[test]
    fn media_source_volume_clamp() {
        let src = MediaSource::new("test.wav").with_volume(5.0);
        assert_eq!(src.volume, 1.0);
        let src = MediaSource::new("test.wav").with_volume(-1.0);
        assert_eq!(src.volume, 0.0);
    }

    #[test]
    fn media_source_as_component() {
        let mut world = crate::World::new();
        let e = world.spawn();
        world
            .insert_component(e, MediaSource::new("cutscene.mp4"))
            .unwrap();
        assert!(world.has_component::<MediaSource>(e));
        let src = world.get_component::<MediaSource>(e).unwrap();
        assert_eq!(src.source, "cutscene.mp4");
    }
}
