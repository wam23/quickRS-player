use super::*;

use qrs_analysis::take::BarResult;

#[test]
fn pocket_summary_is_bass_only_and_needs_enough_notes() {
    let timing = "dragging: 24 ms late on average";
    assert_eq!(
        timing_line(timing, true, 3, 26.0),
        "dragging: 24 ms late on average  ·  pocket spread 26 ms"
    );
    assert_eq!(timing_line(timing, true, 2, 26.0), timing);
    assert_eq!(timing_line("on time", false, 10, 26.0), "on time");
}

#[test]
fn bar_tooltip_reports_signed_pocket_offset() {
    let bar = BarResult {
        number: 3,
        index: 1,
        t0: 0.0,
        t1: 2.0,
        hits: 9,
        wrong: 1,
        missed: 0,
        not_checked: 0,
        extras: 0,
        mean_offset_ms: Some(-23.6),
    };
    assert_eq!(bar_tooltip(&bar, true), "bar 3: 90%  ·  pocket -24 ms");
    assert_eq!(bar_tooltip(&bar, false), "bar 3: 90%  ·  timing -24 ms");
}
