use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Interaction {
    #[serde(with = "time::serde::timestamp")]
    ts: OffsetDateTime,
    event: Event,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind")]
pub enum Event {
    #[serde(rename = "mousemovement")]
    MouseMovement { x: i32, y: i32 },
    #[serde(rename = "mouseclick")]
    MouseClick { up_down: UpDown },
    #[serde(rename = "mouseenter")]
    MouseEnter { in_out: InOut },
    #[serde(rename = "keypress")]
    KeyPress { up_down: UpDown, key: String },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UpDown {
    Up,
    Down,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum InOut {
    In,
    Out,
}

pub struct Score(pub f32);

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

    let mut score_sum = 0f32;
    for &action in actions.iter() {
        let action_score = match action {
            [] => 0f32,
            [_] => 1f32,
            [first, .., last] => match (first, last) {
                (
                    Interaction { ts: ts1, event: Event::MouseClick { up_down: UpDown::Down } },
                    Interaction { ts: ts2, event: Event::MouseClick { up_down: UpDown::Up } },
                )
                | (
                    Interaction { ts: ts1, event: Event::KeyPress { up_down: UpDown::Down, .. } },
                    Interaction { ts: ts2, event: Event::KeyPress { up_down: UpDown::Up, .. } },
                ) => timing_score_for_click(ts1.unix_timestamp(), ts2.unix_timestamp()),
                _ => 1f32,
            },
        };
        score_sum += action_score;
    }

    Score(score_sum / (actions.len() as f32))
}

fn timing_score_for_click(ts1: i64, ts2: i64) -> f32 {
    // TODO: gaussian distribution. https://stackoverflow.com/questions/38846373/how-much-time-should-the-mouse-left-button-be-held
    let res = ts2 - ts1;
    match res {
        2..350 => 1f32,
        _ => 0f32,
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    #[test]
    fn asd() -> anyhow::Result<()> {
        Err(anyhow!("write the tests"))
    }
}
