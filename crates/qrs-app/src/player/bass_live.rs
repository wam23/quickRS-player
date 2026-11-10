use eframe::egui::{self, Align2, Painter, Rect, Stroke, pos2, vec2};
use qrs_analysis::detect::{Judgement, NoteGroup};
use qrs_ui::paint::{self, Fill};
use qrs_ui::theme::t;
use qrs_ui::units::px;

use crate::library::{caps, txt};

const WINDOW_SECONDS: f64 = 6.0;
const WINDOW_NOTES: usize = 8;
const MIN_NOTES: usize = 3;
const METER_RANGE_MS: f64 = 100.0;

pub(super) struct Timing {
    pub mean_ms: f64,
    pub spread_ms: f64,
    pub notes: usize,
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
    Some(Timing {
        mean_ms,
        spread_ms,
        notes: notes.len(),
    })
}

fn meter_x(ms: f64, track: Rect) -> f32 {
    let fraction = ((ms.clamp(-METER_RANGE_MS, METER_RANGE_MS) / METER_RANGE_MS + 1.0) / 2.0) as f32;
    track.left() + fraction * track.width()
}

pub(super) fn draw(p: &Painter, area: Rect, timing: Option<&Timing>) {
    let th = t();
    let width = area.width().min(px(720.0));
    let card = Rect::from_center_size(area.center(), vec2(width, px(82.0)));
    paint::inset(p, card, Some(14.0), 2.0, Fill::Base);

    caps(
        p,
        "BASS POCKET",
        pos2(card.min.x + px(18.0), card.min.y + px(15.0)),
        10.0,
        Align2::LEFT_CENTER,
        None,
    );
    if let Some(timing) = timing {
        let state = if timing.mean_ms.abs() <= qrs_analysis::take::ON_TIME_MS {
            "IN POCKET"
        } else if timing.mean_ms < 0.0 {
            "EARLY"
        } else {
            "LATE"
        };
        let color = if timing.mean_ms.abs() <= qrs_analysis::take::ON_TIME_MS {
            th.hit
        } else if timing.mean_ms.abs() <= qrs_analysis::detect::PERFECT * 1000.0 {
            th.late
        } else {
            th.miss
        };
        txt(
            p,
            &format!("{:+.0} ms", timing.mean_ms),
            pos2(card.min.x + px(150.0), card.min.y + px(15.0)),
            21.0,
            color,
            800,
            Align2::LEFT_CENTER,
            None,
        );
        txt(
            p,
            &format!(
                "{state}  ·  spread {:.0} ms  ·  {} notes",
                timing.spread_ms, timing.notes
            ),
            pos2(card.max.x - px(18.0), card.min.y + px(15.0)),
            11.0,
            th.caption,
            600,
            Align2::RIGHT_CENTER,
            None,
        );
    } else {
        txt(
            p,
            "Play 3 clear notes to measure",
            pos2(card.max.x - px(18.0), card.min.y + px(15.0)),
            12.0,
            th.caption,
            600,
            Align2::RIGHT_CENTER,
            None,
        );
    }

    let track = Rect::from_min_max(
        pos2(card.min.x + px(20.0), card.min.y + px(37.0)),
        pos2(card.max.x - px(20.0), card.min.y + px(52.0)),
    );
    p.rect_filled(track, track.height() / 2.0, th.unlit.gamma_multiply(0.65));
    p.rect_stroke(
        track,
        track.height() / 2.0,
        Stroke::new(px(1.0), th.line),
        egui::StrokeKind::Inside,
    );
    for mark in [-100.0, -50.0, 0.0, 50.0, 100.0] {
        let x = meter_x(mark, track);
        p.line_segment(
            [pos2(x, track.min.y - px(3.0)), pos2(x, track.max.y + px(3.0))],
            Stroke::new(px(if mark == 0.0 { 2.0 } else { 1.0 }), th.caption),
        );
    }
    if let Some(timing) = timing {
        let color = if timing.mean_ms.abs() <= qrs_analysis::take::ON_TIME_MS {
            th.hit
        } else if timing.mean_ms.abs() <= qrs_analysis::detect::PERFECT * 1000.0 {
            th.late
        } else {
            th.miss
        };
        let a = meter_x(timing.mean_ms - timing.spread_ms, track);
        let b = meter_x(timing.mean_ms + timing.spread_ms, track);
        let spread = Rect::from_min_max(pos2(a, track.min.y), pos2(b.max(a + px(2.0)), track.max.y));
        p.rect_filled(spread, track.height() / 2.0, color.gamma_multiply(0.55));
        let x = meter_x(timing.mean_ms, track);
        p.line_segment(
            [pos2(x, track.min.y - px(5.0)), pos2(x, track.max.y + px(5.0))],
            Stroke::new(px(3.0), color),
        );
        p.circle_filled(pos2(x, track.center().y), px(5.0), color);
    }
    txt(
        p,
        "EARLY  −100 ms",
        pos2(track.min.x, card.max.y - px(11.0)),
        9.0,
        th.caption,
        600,
        Align2::LEFT_CENTER,
        None,
    );
    txt(
        p,
        "ON TIME",
        pos2(track.center().x, card.max.y - px(11.0)),
        9.0,
        th.text_2,
        700,
        Align2::CENTER_CENTER,
        None,
    );
    txt(
        p,
        "LATE  +100 ms",
        pos2(track.max.x, card.max.y - px(11.0)),
        9.0,
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
        assert_eq!(result.notes, 8);
    }

    #[test]
    fn meter_centres_zero_and_clamps_both_ends() {
        let track = Rect::from_min_max(pos2(10.0, 0.0), pos2(210.0, 20.0));
        assert_eq!(meter_x(0.0, track), 110.0);
        assert_eq!(meter_x(-200.0, track), 10.0);
        assert_eq!(meter_x(200.0, track), 210.0);
    }
}
