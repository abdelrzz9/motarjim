//! Structured CSS Grid types for the Motarjim compiler.

#![allow(missing_docs)]

/// A parsed grid template (e.g., `grid-template-columns: 1fr 200px auto`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct GridTemplate {
    /// The tracks defined in this template.
    pub tracks: Vec<GridTrack>,
}

/// A single track in a grid template.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum GridTrack {
    /// Fixed length in pixels.
    Fixed(f64),
    /// Fractional unit (`1fr`, `2fr`).
    Fr(f64),
    /// `minmax(min, max)` track.
    MinMax(Box<GridTrack>, Box<GridTrack>),
    /// `auto` keyword.
    Auto,
    /// `min-content` keyword.
    MinContent,
    /// `max-content` keyword.
    MaxContent,
    /// `fit-content(length)` track.
    FitContent(f64),
    /// `repeat(count, tracks)` — repeats a pattern.
    Repeat(u32, Vec<GridTrack>),
}

/// A grid line placement (e.g., `grid-column: 1 / 3`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct GridPlacement {
    /// The starting line.
    pub line: GridLine,
    /// Optional span (e.g., `span 2`).
    pub span: Option<u32>,
}

/// A grid line reference.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum GridLine {
    /// `auto` placement.
    Auto,
    /// Named line (e.g., `"start"`).
    Named(String),
    /// Numeric line (1-based).
    Number(u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_template_basic() {
        let t = GridTemplate {
            tracks: vec![GridTrack::Fr(1.0), GridTrack::Fr(1.0), GridTrack::Fr(1.0)],
        };
        assert_eq!(t.tracks.len(), 3);
        assert_eq!(t.tracks[0], GridTrack::Fr(1.0));
    }

    #[test]
    fn test_grid_track_mixed() {
        let t = GridTemplate {
            tracks: vec![
                GridTrack::Fixed(200.0),
                GridTrack::Auto,
                GridTrack::Fr(1.0),
            ],
        };
        assert_eq!(t.tracks.len(), 3);
        assert_eq!(t.tracks[0], GridTrack::Fixed(200.0));
        assert_eq!(t.tracks[2], GridTrack::Fr(1.0));
    }

    #[test]
    fn test_grid_placement() {
        let p = GridPlacement {
            line: GridLine::Number(1),
            span: Some(2),
        };
        assert_eq!(p.line, GridLine::Number(1));
        assert_eq!(p.span, Some(2));
    }
}
