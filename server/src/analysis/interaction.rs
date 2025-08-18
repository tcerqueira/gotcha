use std::collections::{HashMap, VecDeque};

use fitting::Gaussian;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Stores an interaction event at a certain point in time.
/// An interaction refers to actions of the user with the computer and currently it includes
/// any mouse movement, mouse clicks, mouse enter and exit out of the target and key presses.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Interaction {
    #[serde(with = "time::serde::timestamp::milliseconds")]
    ts: OffsetDateTime,
    event: Event,
}

/// Describes the kind of event for the interaction.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind")]
enum Event {
    #[serde(rename = "mousemovement")]
    MouseMovement { x: i32, y: i32 },
    #[serde(rename = "mouseclick")]
    MouseClick {
        #[serde(rename = "mouse")]
        up_down: UpDown,
    },
    #[serde(rename = "mouseenter")]
    MouseEnter {
        #[serde(rename = "mouse")]
        in_out: InOut,
    },
    #[serde(rename = "keypress")]
    KeyPress {
        #[serde(rename = "keyMove")]
        up_down: UpDown,
        key: String,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum UpDown {
    Up,
    Down,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum InOut {
    In,
    Out,
}

/// A score from 0f to 1f of how confident thinks a human interacted with the computer.
pub struct Score(pub f32);

/// Takes a group of ordered interactions and computes the score.
pub fn interaction_analysis(interactions: &[Interaction]) -> Score {
    let mut actions: Vec<&[Interaction]> = vec![];
    let mut events_stacks = HashMap::<String, VecDeque<usize>>::new();
    let mut curr_action = 0;
    for (i, it) in interactions.iter().enumerate() {
        // generic action delimited by any interaction besides movement
        match it.event {
            Event::MouseMovement { .. } => {}
            _ => {
                actions.push(&interactions[curr_action..=i]);
                curr_action = i;
            }
        }
        // actions delimited by events of the same nature: click, key press, mouse enter
        let key_to_push = match it.event {
            Event::MouseEnter { in_out: InOut::In } => Some("mouseenter"),
            Event::MouseClick { up_down: UpDown::Down } => Some("click"),
            Event::KeyPress { ref key, up_down: UpDown::Down } => Some(key.as_str()),
            _ => None,
        };
        if let Some(key) = key_to_push {
            events_stacks.entry(key.into()).or_default().push_back(i);
            // we know we matched a push, no point trying to match a pop in the next step
            continue;
        }

        let key_to_pop = match it.event {
            Event::MouseEnter { in_out: InOut::Out } => Some("mouseenter"),
            Event::MouseClick { up_down: UpDown::Up } => Some("click"),
            Event::KeyPress { ref key, up_down: UpDown::Up } => Some(key.as_str()),
            _ => None,
        };
        if let Some(last_i) =
            key_to_pop.and_then(|key| events_stacks.entry(key.into()).or_default().pop_back())
        {
            actions.push(&interactions[last_i..=i])
        }
    }

    let score_sum: f32 = actions
        .par_iter()
        .map(|&action| match action {
            [] => 0.,
            [_] => 0.5,
            [first, .., last] => match (first, last) {
                (
                    Interaction { ts: ts1, event: Event::MouseClick { up_down: UpDown::Down } },
                    Interaction { ts: ts2, event: Event::MouseClick { up_down: UpDown::Up } },
                )
                | (
                    Interaction { ts: ts1, event: Event::KeyPress { up_down: UpDown::Down, .. } },
                    Interaction { ts: ts2, event: Event::KeyPress { up_down: UpDown::Up, .. } },
                ) => timing_score_for_click(
                    (ts1.unix_timestamp_nanos() / 1_000_000) as i64,
                    (ts2.unix_timestamp_nanos() / 1_000_000) as i64,
                ),
                _ => 0.5,
            },
        })
        .sum();

    Score(match actions.len() {
        0 => 0.,
        len => score_sum / (len as f32),
    })
}

// Two Gaussian curves for track pad and mouse click timings overlapped
fn timing_score_for_click(ts1: i64, ts2: i64) -> f32 {
    match ts2 - ts1 {
        2..15 => 1.,
        t @ 15..220 => {
            let f_trackpad = Gaussian::new(150., 60., 1.);
            let f_mouse = Gaussian::new(100., 45., 1.);
            f32::max(f_trackpad.value(t as f32), f_mouse.value(t as f32))
        }
        220.. => 0.5,
        _ => 0.,
    }
    .min(1f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    use time::{Duration, OffsetDateTime};

    #[test]
    fn positive_score() -> anyhow::Result<()> {
        let now = OffsetDateTime::now_utc();
        let interactions = vec![
            Interaction {
                ts: now + Duration::milliseconds(50),
                event: Event::MouseEnter { in_out: InOut::In },
            },
            Interaction {
                ts: now + Duration::milliseconds(100),
                event: Event::MouseMovement { x: 0, y: 10 },
            },
            Interaction {
                ts: now + Duration::milliseconds(150),
                event: Event::MouseMovement { x: 10, y: 10 },
            },
            Interaction {
                ts: now + Duration::milliseconds(200),
                event: Event::MouseClick { up_down: UpDown::Down },
            },
            Interaction {
                ts: now + Duration::milliseconds(260),
                event: Event::MouseClick { up_down: UpDown::Up },
            },
        ];

        let Score(score) = interaction_analysis(&interactions);
        assert!(dbg!(score) >= 0.5);

        Ok(())
    }
}
