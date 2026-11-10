use super::{Ctx, Ev, EventKind, EventResult, TakeReport, Techs};
use crate::detect::{self, ATTACK_SKIP};
use crate::onsets::TakeOnset;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoldIssue {
    CutShort,
    Ringing,
}

impl HoldIssue {
    pub fn key(self) -> &'static str {
        self.words().0
    }

    fn words(self) -> (&'static str, &'static str) {
        match self {
            HoldIssue::CutShort => ("cut_short", "faded before their charted length"),
            HoldIssue::Ringing => ("ringing", "rang past their charted length; mute sooner"),
        }
    }

    fn problem(self, location: &str, count: usize) -> String {
        let noun = if count == 1 { "bass note" } else { "bass notes" };
        format!("{location}: {count} {noun} {}.", self.words().1)
    }
}

/// Only long, isolated, clear bass notes are checked; probes contaminated by another attack are ignored.
pub(super) fn hold_issue(
    ctx: &mut Ctx,
    event: &Ev,
    onset: usize,
    expected_duration: usize,
    previous_end: Option<usize>,
    next_onset: Option<usize>,
    onsets: &[TakeOnset],
) -> Option<HoldIssue> {
    const MIN_SUSTAIN: f64 = 0.55;
    const RELEASE_TOLERANCE: f64 = 0.15;
    const ATTACK_GUARD: f64 = 0.03;

    if event.kind != EventKind::Note
        || event.notes.len() != 1
        || event.legato()
        || event.sustain() < MIN_SUSTAIN
    {
        return None;
    }
    let note = &event.notes[0];
    let excluded = Techs::FRET_HAND_MUTE
        | Techs::PALM_MUTE
        | Techs::BEND
        | Techs::SLIDE
        | Techs::UNPITCHED_SLIDE
        | Techs::VIBRATO
        | Techs::HAMMER_ON
        | Techs::PULL_OFF
        | Techs::TAP
        | Techs::LINK_NEXT;
    if note.linked || (note.techs | event.techs) & excluded != Techs::NONE {
        return None;
    }

    let rate = ctx.rate as f64;
    let window = (rate
        * if note.pos.midi < 40 {
            0.17
        } else {
            detect::BASS_WINDOW
        }) as usize;
    let tolerance = (rate * RELEASE_TOLERANCE) as usize;
    let attack_guard = (rate * ATTACK_GUARD) as usize;
    let expected_end = onset.saturating_add(expected_duration);
    if previous_end.is_some_and(|end| end.saturating_add(window + tolerance) > onset) {
        return None;
    }
    let onset_probe = onset + (ATTACK_SKIP * rate) as usize;
    if !note_present(ctx, note.pos.midi, onset_probe, window)? {
        return None;
    }

    let early_start = expected_end.saturating_sub(window + tolerance);
    let early_clear = next_onset.is_none_or(|next| next >= early_start + window)
        && !onset_overlaps(onsets, early_start, early_start + window, attack_guard);
    if early_clear && note_present(ctx, note.pos.midi, early_start, window) == Some(false) {
        return Some(HoldIssue::CutShort);
    }

    let late_start = expected_end.saturating_add(tolerance);
    let late_clear = next_onset.is_none_or(|next| next >= late_start + window)
        && !onset_overlaps(onsets, late_start, late_start + window, attack_guard);
    if late_clear && note_present(ctx, note.pos.midi, late_start, window) == Some(true) {
        return Some(HoldIssue::Ringing);
    }
    None
}

fn note_present(ctx: &mut Ctx, midi: i32, start: usize, len: usize) -> Option<bool> {
    let mag = ctx.spec(start, len)?;
    Some(detect::fundamental_present_in(
        mag.as_slice(),
        ctx.cache.binhz,
        midi,
        None,
        true,
    ))
}

fn onset_overlaps(onsets: &[TakeOnset], start: usize, end: usize, guard: usize) -> bool {
    let start = start.saturating_sub(guard);
    let end = end.saturating_add(guard);
    onsets.iter().any(|o| start <= o.sample && o.sample <= end)
}

pub(super) fn problems(report: &TakeReport) -> Vec<(f64, String)> {
    let mut out = Vec::new();
    for issue in [HoldIssue::CutShort, HoldIssue::Ringing] {
        let events: Vec<&EventResult> = report.events.iter().filter(|e| e.hold == Some(issue)).collect();
        if events.is_empty() {
            continue;
        }
        let bars: Vec<usize> = events
            .iter()
            .map(|e| report.bars[e.bar].number.max(0) as usize)
            .collect();
        let count = events.len();
        let location = super::capital(&super::list_bars(&bars));
        out.push((1.25 * count as f64, issue.problem(&location, count)));
    }
    out
}

#[cfg(test)]
mod tests;
