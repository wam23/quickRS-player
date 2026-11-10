//! The live pocket: how far the last few right notes sat from the beat, while a bass part plays.

use eframe::egui::{Align2, Color32, Painter, Rect, pos2, vec2};
use qrs_analysis::detect::{Judgement, NoteGroup};
use qrs_ui::paint;
use qrs_ui::theme::t;
use qrs_ui::units::px;

use crate::library::{caps, txt};

const WINDOW_SECONDS: f64 = 6.0;
const WINDOW_NOTES: usize = 8;
const MIN_NOTES: usize = 3;

pub(super) struct Timing {
    pub mean_ms: f64,
    pub spread_ms: f64,
}

pub(super) fn timing(groups: &[NoteGroup], now: f64) -> Option<Timing> {
    let mut notes: Vec<(f64, f64)> = groups
        .iter()
        .filter(|g| matches!(g.state, Some(Judgement::Hit | Judgement::Early | Judgement::Late)))
        .filter(|g| g.dt.is_finite() && g.judged_wall.is_finite())
        .filter_map(|g| {
            (0.0..=WINDOW_SECONDS)
                .contains(&(now - g.judged_wall))
                .then_some((g.judged_wall, g.dt * 1000.0))
        })
        .collect();
    notes.sort_by(|a, b| a.0.total_cmp(&b.0));
    let notes = &notes[notes.len().saturating_sub(WINDOW_NOTES)..];
    if notes.len() < MIN_NOTES {
        return None;
    }
    let mean_ms = notes.iter().map(|n| n.1).sum::<f64>() / notes.len() as f64;
    let spread_ms = (notes.iter().map(|n| (n.1 - mean_ms).powi(2)).sum::<f64>() / notes.len() as f64).sqrt();
    Some(Timing { mean_ms, spread_ms })
}

/// The header strip's one line: `pocket +12 ±8 ms`.
pub(super) fn line(timing: &Timing) -> String {
    format!("pocket {:+.0} ±{:.0} ms", timing.mean_ms, timing.spread_ms)
}

/// The mean's colour: in pocket, within a perfect's reach, or off.
pub(super) fn mood(mean_ms: f64) -> Color32 {
    let th = t();
    if mean_ms.abs() <= qrs_analysis::take::ON_TIME_MS {
        th.hit
    } else if mean_ms.abs() <= qrs_analysis::detect::PERFECT * 1000.0 {
        th.late
    } else {
        th.miss
    }
}

/// The full screen's plate under the streak: the mean big, the spread beside it.
pub(super) fn draw(p: &Painter, well: Rect, timing: &Timing) {
    let th = t();
    let plate = Rect::from_min_size(
        pos2(well.min.x + px(16.0), well.min.y + px(58.0)),
        vec2(px(190.0), px(50.0)),
    );
    paint::floating(p, plate, Some(12.0));
    caps(
        p,
        "POCKET",
        pos2(plate.min.x + px(14.0), plate.min.y + px(15.0)),
        9.5,
        Align2::LEFT_CENTER,
        None,
    );
    txt(
        p,
        &format!("{:+.0} ms", timing.mean_ms),
        pos2(plate.min.x + px(14.0), plate.min.y + px(34.0)),
        18.0,
        mood(timing.mean_ms),
        800,
        Align2::LEFT_CENTER,
        None,
    );
    txt(
        p,
        &format!("±{:.0} ms", timing.spread_ms),
        pos2(plate.max.x - px(14.0), plate.min.y + px(34.0)),
        11.0,
        th.caption,
        600,
        Align2::RIGHT_CENTER,
        None,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn group(state: Judgement, dt_ms: f64, judged_wall: f64) -> NoteGroup {
        NoteGroup {
            t: 0.0,
            midis: vec![],
            strings: vec![],
            muted: false,
            legato: false,
            state: Some(state),
            dt: dt_ms / 1000.0,
            judged_wall,
            last_check: 0.0,
            repeats: HashSet::new(),
            ringing: vec![],
        }
    }

    #[test]
    fn live_pocket_uses_recent_right_notes_and_needs_three() {
        let few = [
            group(Judgement::Hit, 10.0, 9.0),
            group(Judgement::Late, 20.0, 9.5),
        ];
        assert!(timing(&few, 10.0).is_none());

        let mut groups: Vec<_> = (0..10)
            .map(|i| group(Judgement::Hit, i as f64, 5.0 + i as f64 * 0.5))
            .collect();
        groups.push(group(Judgement::Miss, 1000.0, 9.9));
        groups.push(group(Judgement::Hit, -1000.0, 0.0));

        let result = timing(&groups, 10.0).unwrap();
        assert_eq!(result.mean_ms, 5.5); // the latest eight hits: 2..9 ms
        assert!((result.spread_ms - 2.291).abs() < 0.001);
    }

    #[test]
    fn line_names_the_window() {
        let t = Timing {
            mean_ms: 12.4,
            spread_ms: 7.6,
        };
        assert_eq!(line(&t), "pocket +12 ±8 ms");
    }
}
