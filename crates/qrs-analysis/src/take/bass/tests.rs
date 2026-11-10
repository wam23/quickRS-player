use super::*;

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

use qrs_formats::model::{Arrangement, Beat, Note};

use crate::onsets::TakeOnset;
use crate::spectrum::Spectra;
use crate::take::{Ctx, SpecCache, TakeInput, TakeTiming, analyse};

const RATE: u32 = 48_000;

fn bass_recording(sounded: f64) -> (Arrangement, Vec<f32>) {
    let mut arr = Arrangement::new("Bass");
    arr.is_bass = true;
    arr.beats = (0..=6)
        .map(|i| Beat {
            time: i as f64 * 0.5,
            measure: if i % 4 == 0 { i / 4 + 1 } else { -1 },
        })
        .collect();
    arr.notes.push(Note {
        sustain: 0.8,
        ..Note::new(0.5, 2, 2) // E2 on the D string
    });

    let mut samples = vec![0.0; 3 * RATE as usize];
    let (start, end) = (0.5 * RATE as f64, (0.5 + sounded) * RATE as f64);
    let frequency = crate::pitch::midi_to_hz(40.0);
    for i in start as usize..end as usize {
        let t = (i as f64 - start) / RATE as f64;
        let attack = (t / 0.008).min(1.0);
        let release = ((end - i as f64) / (0.008 * RATE as f64)).clamp(0.0, 1.0);
        let phase = std::f64::consts::TAU * frequency * t;
        let wave = (1..=6).map(|h| (phase * h as f64).sin() / h as f64).sum::<f64>();
        samples[i] = (0.25 * attack * release * wave) as f32;
    }

    (arr, samples)
}

fn bass_hold_take(sounded: f64) -> TakeReport {
    let (arr, samples) = bass_recording(sounded);
    let input = TakeInput {
        arr,
        from: 0.0,
        to: 2.5,
        samples,
        timing: TakeTiming::linear(RATE, 0.0, 1.0),
    };
    analyse(&input, &|_| {}, &AtomicBool::new(false)).unwrap()
}

#[test]
fn bass_hold_feedback_flags_short_and_ringing_notes_but_not_a_clean_release() {
    let short = bass_hold_take(0.35);
    assert_eq!(short.events[0].status, crate::take::Status::Hit);
    assert_eq!(short.events[0].hold, Some(HoldIssue::CutShort));
    assert!(short.problems.iter().any(|p| p.contains("faded before")));
    assert_eq!(short.to_json()["events"][0]["hold"], "cut_short");
    assert_eq!(short.to_json()["song"]["is_bass"], true);

    let held = bass_hold_take(0.8);
    assert_eq!(held.events[0].status, crate::take::Status::Hit);
    assert_eq!(held.events[0].hold, None);

    let ringing = bass_hold_take(1.2);
    assert_eq!(ringing.events[0].status, crate::take::Status::Hit);
    assert_eq!(ringing.events[0].hold, Some(HoldIssue::Ringing));
    assert!(ringing.problems.iter().any(|p| p.contains("rang past")));
}

#[test]
fn attacks_inside_a_probe_window_make_it_ambiguous() {
    let onsets = [TakeOnset {
        sample: 1_000,
        strength: 1.0,
        attack: false,
    }];
    assert!(onset_overlaps(&onsets, 900, 1_100, 20));
    assert!(!onset_overlaps(&onsets, 1_100, 1_300, 20));
}

#[test]
fn a_charted_overlap_is_not_reported_as_a_hold_issue() {
    let (arr, samples) = bass_recording(1.2);
    let event = crate::take::chart_events(&arr).remove(0);
    let mut ctx = Ctx {
        x: &samples,
        rate: RATE,
        sp: Spectra::new(),
        cache: SpecCache {
            map: HashMap::new(),
            binhz: 0.0,
        },
    };
    let onset = (0.5 * RATE as f64) as usize;
    let overlapping_previous_end = onset - RATE as usize / 10;
    let duration = (0.8 * RATE as f64) as usize;
    assert_eq!(
        hold_issue(
            &mut ctx,
            &event,
            onset,
            duration,
            Some(overlapping_previous_end),
            None,
            &[]
        ),
        None
    );
}

#[test]
fn cached_spectra_restore_their_bin_width() {
    let samples = vec![0.1; 10_000];
    let mut ctx = Ctx {
        x: &samples,
        rate: RATE,
        sp: Spectra::new(),
        cache: SpecCache {
            map: HashMap::new(),
            binhz: 0.0,
        },
    };
    ctx.spec(0, 512).unwrap();
    let short_bins = ctx.cache.binhz;
    ctx.spec(512, 5_000).unwrap();
    assert_ne!(ctx.cache.binhz, short_bins);
    ctx.spec(0, 512).unwrap();
    assert_eq!(ctx.cache.binhz, short_bins);
}
