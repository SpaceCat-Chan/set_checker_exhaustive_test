use std::collections::BTreeSet;
use std::collections::HashSet;
use std::time::Duration;
use std::time::Instant;

use indicatif::ProgressIterator;
use indicatif::ProgressStyle;
use literally::bset;

fn main() {
    let mut test_cases: Vec<Vec<BTreeSet<u8>>> =
        serde_json_lenient::from_str(include_str!("cases.txt")).unwrap();

    for test_case in &mut test_cases {
        test_case.extend([bset! {0}, bset! {1}, bset! {2}, bset! {3}]);
    }

    let progress = indicatif::ProgressBar::new(test_cases.len() as _);
    progress.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}, {eta_precise} left] [{bar:60.green/green}] {pos:>7}/{len:7}",
        )
        .unwrap()
        .progress_chars("━╸ "),
    );

    let mut agree_fails = 0;
    let mut agree_success = 0;
    let mut disagree_backtrack_fail = 0;
    let mut disagree_backtrack_success = 0;
    let mut total_hue_time = Duration::ZERO;
    let mut total_back_time = Duration::ZERO;
    for (index, test_case) in test_cases.into_iter().enumerate() {
        progress.inc(1);
        let hue_time = Instant::now();
        let heuristic_res = check_signals_heuristic(&test_case[..]);
        let back_time = Instant::now();
        let backtracker_res = check_signals_backtracker(&test_case[..]);
        let done_time = Instant::now();
        total_hue_time += back_time - hue_time;
        total_back_time += done_time - back_time;
        if heuristic_res == backtracker_res {
            if backtracker_res {
                agree_success += 1;
            } else {
                agree_fails += 1;
            }
        } else if backtracker_res {
            progress.println(format!(
                "disagreement on case {index}, backtracker says it should have worked"
            ));
            disagree_backtrack_success += 1;
        } else {
            progress.println(format!(
                "disagreement on case {index}, backtracker says it should have failed"
            ));
            disagree_backtrack_fail += 1;
        }
    }
    println!("agreements: {} ({agree_fails} fail, {agree_success} succeed), disagreements: {} ({disagree_backtrack_fail} should have failed, {disagree_backtrack_success} should have succeeded)", agree_fails + agree_success, disagree_backtrack_fail + disagree_backtrack_success);
    println!(
        "hueristic took {}s, backtracker took {}s",
        total_hue_time.as_secs_f64(),
        total_back_time.as_secs_f64()
    );
}

const a_capacity: u8 = 11;
const b_capacity: u8 = 12;
const c_capacity: u8 = 13;
const d_capacity: u8 = 10;

fn check_signals_heuristic(signals: &[BTreeSet<u8>]) -> bool {
    // a quick thing to imitate how lofty's hashing solution works
    // return signals.iter().map(|set| set.len()).sum::<usize>() <= 46;

    let (mut a, mut b, mut c, mut d) = (0, 0, 0, 0);
    let (mut ac, mut ad, mut bc, mut bd) = (0, 0, 0, 0);

    for signal in signals {
        for set in signal {
            match set {
                0 => ac += 1,
                1 => ad += 1,
                2 => bc += 1,
                3 => bd += 1,
                _ => unreachable!(),
            }
        }
        match signal.len() {
            1 => {}
            2 => match (signal.first().unwrap(), signal.last().unwrap()) {
                (0, 1) => a = a_capacity.min(a + 1),
                (0, 2) => c = c_capacity.min(c + 1),
                (0, 3) | (1, 2) => {}
                (1, 3) => d = d_capacity.min(d + 1),
                (2, 3) => b = b_capacity.min(b + 1),
                _ => unreachable!(),
            },
            3 => match (
                signal.contains(&0),
                signal.contains(&1),
                signal.contains(&2),
                signal.contains(&3),
            ) {
                (false, _, _, _) | (_, false, _, _) => b = b_capacity.min(b + 1),
                (_, _, false, _) | (_, _, _, false) => a = a_capacity.min(a + 1),
                _ => unreachable!(),
            },
            4 => {
                a = a_capacity.min(a + 1);
                c = c_capacity.min(c + 1);
            }
            _ => unreachable!(),
        }
        if a > a_capacity
            || b > b_capacity
            || c > c_capacity
            || d > d_capacity
            || ac > a_capacity + c_capacity
            || ad > a_capacity + d_capacity
            || bc > b_capacity + c_capacity
            || bd > b_capacity + d_capacity
            || ac + ad - a > a_capacity + c_capacity + d_capacity
            || bc + bd - b > b_capacity + c_capacity + d_capacity
            || ac + bc - c > a_capacity + b_capacity + c_capacity
            || ad + bd - d > a_capacity + b_capacity + d_capacity
            || ac + ad + bc + bd - a - b - d - c > a_capacity + b_capacity + c_capacity + d_capacity
        {
            return false;
        }
    }
    true
}

fn check_signals_backtracker(signals: &[BTreeSet<u8>]) -> bool {
    //unsafe {
    //    FAILS = 0;
    //    NODES = 0;
    //}
    //println!("starting backtracker on {} signals", signals.len());
    //let now = std::time::Instant::now();
    let res = check_signals_backtracker_recurse(signals, (0, 0, 0, 0), &mut HashSet::new());
    //println!(
    //    "backtracker took {}s with {} failures over {} nodes",
    //    now.elapsed().as_secs_f32(),
    //    unsafe { FAILS },
    //    unsafe { NODES }
    //);
    res
}

static mut FAILS: usize = 0;
static mut NODES: usize = 0;

fn check_signals_backtracker_recurse(
    signals: &[BTreeSet<u8>],
    current_counts: (u8, u8, u8, u8), // (A, B, C, D)
    seen_states: &mut HashSet<((u8, u8, u8, u8), usize)>,
) -> bool {
    unsafe {
        NODES += 1;
    }
    if current_counts.0 > a_capacity
        || current_counts.1 > b_capacity
        || current_counts.2 > c_capacity
        || current_counts.3 > d_capacity
    {
        unsafe {
            FAILS += 1;
        }
        return false;
    }
    if seen_states.contains(&(current_counts, signals.len())) {
        unsafe {
            FAILS += 1;
        }
        return false;
    }
    match signals.split_first() {
        None => true,
        Some((sets, rest)) => {
            let mut new_to_test = match (
                sets.contains(&0),
                sets.contains(&1),
                sets.contains(&2),
                sets.contains(&3),
            ) {
                (false, false, false, false) => unreachable!(),

                (true, false, false, false) => vec![(1, 0, 0, 0), (0, 0, 1, 0)],
                (false, true, false, false) => vec![(1, 0, 0, 0), (0, 0, 0, 1)],
                (false, false, true, false) => vec![(0, 1, 0, 0), (0, 0, 1, 0)],
                (false, false, false, true) => vec![(0, 1, 0, 0), (0, 0, 0, 1)],

                (true, true, false, false) => vec![(1, 0, 0, 0), (0, 0, 1, 1)],
                (false, false, true, true) => vec![(0, 1, 0, 0), (0, 0, 1, 1)],
                (true, false, true, false) => vec![(0, 0, 1, 0), (1, 1, 0, 0)],
                (false, true, false, true) => vec![(0, 0, 0, 1), (1, 1, 0, 0)],

                (true, false, false, true) => {
                    vec![(1, 1, 0, 0), (1, 0, 0, 1), (0, 1, 1, 0), (0, 0, 1, 1)]
                }
                (false, true, true, false) => {
                    vec![(1, 1, 0, 0), (1, 0, 1, 0), (0, 1, 0, 1), (0, 0, 1, 1)]
                }

                (true, true, true, false) => vec![(1, 1, 0, 0), (1, 0, 1, 0), (0, 0, 1, 1)],
                (true, true, false, true) => vec![(1, 1, 0, 0), (1, 0, 0, 1), (0, 0, 1, 1)],
                (true, false, true, true) => vec![(1, 1, 0, 0), (0, 1, 1, 0), (0, 0, 1, 1)],
                (false, true, true, true) => vec![(1, 1, 0, 0), (0, 1, 0, 1), (0, 0, 1, 1)],

                (true, true, true, true) => vec![(1, 1, 0, 0), (0, 0, 1, 1)],
            };
            for new in &mut new_to_test {
                new.0 += current_counts.0;
                new.1 += current_counts.1;
                new.2 += current_counts.2;
                new.3 += current_counts.3;
            }
            for new_amounts in new_to_test {
                if check_signals_backtracker_recurse(rest, new_amounts, seen_states) {
                    return true;
                }
            }
            seen_states.insert((current_counts, signals.len()));
            unsafe {
                FAILS += 1;
            }
            false
        }
    }
}
