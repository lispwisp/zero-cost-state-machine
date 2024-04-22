use internal::{frame, frames};
use internal::{state_id, transition_id};
use maplit::{btreemap, btreeset};
use crate::{Frame, Frames, StateId, TransitionId, Diagram, StateStereoType};
use std::collections::VecDeque;
use pretty_assertions::assert_eq;
use internal::StateStereoType::*;

use crate::{human_readable_error, mermaid};
#[test]
fn empty() -> anyhow::Result<()> {
    let data = r#""#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert!(diagram.is_empty());
    Ok(())
}

#[test]
fn newline() -> anyhow::Result<()> {
    let data = r#"
    
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert!(diagram.is_empty());
    Ok(())
}

#[test]
fn title() -> anyhow::Result<()> {
    let data = r#"
            ---
            title: Foo
            ---
            stateDiagram-v2
        "#;
    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn comment() -> anyhow::Result<()> {
    let data = r#"
            stateDiagram-v2
            %% comment comment comment
        "#;
    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn symbol_substituion() -> anyhow::Result<()> {
    let data = r#"
            stateDiagram-v2
            state fork_state <<fork>>
              [*] --> fork_state
              fork_state --> State2
              fork_state --> State3
            
            state join_state &lt;&lt;join>> %% &lt; instead of <
              State2 --> join_state
              State3 --> join_state
              join_state --> State4
              State4 --> [*]
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_stereotype: btreemap! {
                state_id!["fork_state"] => Fork,
                state_id!["join_state"] => Join,
            },
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id![End] => state_id![],
                state_id!["State2"] => state_id![],
                state_id!["State3"] => state_id![],
                state_id!["State4"] => state_id![],
                state_id!["fork_state"] => state_id![],
                state_id!["join_state"] => state_id![]
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id![End],
                    state_id!["State2"],
                    state_id!["State3"],
                    state_id!["State4"],
                    state_id!["fork_state"],
                    state_id!["join_state"],
                }
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset!{transition_id!{[Start]->["fork_state"]},},
                state_id!["State2"] => btreeset!{transition_id!{["State2"]->["join_state"]},},
                state_id!["State3"] => btreeset!{transition_id!{["State3"]->["join_state"]},},
                state_id!["State4"] => btreeset!{transition_id!{["State4"]->[End]},},
                state_id!["fork_state"] => btreeset!{
                    transition_id!{["fork_state"]->["State2"]},
                    transition_id!{["fork_state"]->["State3"]},
                },
                state_id!["join_state"] => btreeset!{
                    transition_id!{["join_state"]->["State4"]},
                },
            },
            state_transition_in: btreemap! {
                state_id![End] => btreeset!{transition_id!{["State4"]->[End]},},
                state_id!["State2"] => btreeset!{transition_id!{["fork_state"]->["State2"]},},
                state_id!["State3"] => btreeset!{transition_id!{["fork_state"]->["State3"]},},
                state_id!["State4"] => btreeset!{transition_id!{["join_state"]->["State4"]},},
                state_id!["fork_state"] => btreeset!{
                    transition_id!{[Start]->["fork_state"]},
                },
                state_id!["join_state"] => btreeset!{
                    transition_id!{["State2"]->["join_state"]},
                    transition_id!{["State3"]->["join_state"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start] -> ["fork_state"]} => state_id![Start],
                transition_id!{["State2"] -> ["join_state"]} => state_id!["State2"],
                transition_id!{["State3"] -> ["join_state"]} => state_id!["State3"],
                transition_id!{["State4"] -> [End]} => state_id!["State4"],
                transition_id!{["fork_state"] -> ["State2"]} => state_id!["fork_state"],
                transition_id!{["fork_state"] -> ["State3"]} => state_id!["fork_state"],
                transition_id!{["join_state"] -> ["State4"]} => state_id!["join_state"],
            },
            transition_to: btreemap! {
                transition_id!{[Start] -> ["fork_state"]} => state_id!["fork_state"],
                transition_id!{["State2"] -> ["join_state"]} => state_id!["join_state"],
                transition_id!{["State3"] -> ["join_state"]} => state_id!["join_state"],
                transition_id!{["State4"] -> [End]} => state_id![End],
                transition_id!{["fork_state"] -> ["State2"]} => state_id!["State2"],
                transition_id!{["fork_state"] -> ["State3"]} => state_id!["State3"],
                transition_id!{["join_state"] -> ["State4"]} => state_id!["State4"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn direction() -> anyhow::Result<()> {
    let data = r#"
            stateDiagram-v2
            direction LR
            [*] --> A
            A --> B
            B --> C
            state B {
              direction LR
              a --> b
            }
            B --> D
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    dbg!(input);
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id!["A"] => state_id![],
                state_id!["B"] => state_id![],
                state_id!["B","a"] => state_id!["B"],
                state_id!["B","b"] => state_id!["B"],
                state_id!["C"] => state_id![],
                state_id!["D"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id!["A"],
                    state_id!["B"],
                    state_id!["C"],
                    state_id!["D"],
                },
                state_id!["B"] => btreeset! {
                    state_id!["B","a"],
                    state_id!["B","b"]
                }
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset!{transition_id!{[Start] -> ["A"]},},
                state_id!["A"] => btreeset!{transition_id!{["A"] -> ["B"]},},
                state_id!["B"] => btreeset!{transition_id!{["B"] -> ["C"]},transition_id!{["B"] -> ["D"]},},
                state_id!["B","a"] => btreeset!{transition_id!{["B","a"] -> ["B","b"]},},
            },
            state_transition_in: btreemap! {
                state_id!["A"] => btreeset!{transition_id!{[Start] -> ["A"]},},
                state_id!["B"] => btreeset!{transition_id!{["A"] -> ["B"]},},
                state_id!["B","b"] => btreeset!{transition_id!{["B","a"] -> ["B","b"]},},
                state_id!["C"] => btreeset!{transition_id!{["B"] -> ["C"]},},
                state_id!["D"] => btreeset!{transition_id!{["B"] -> ["D"]},},
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["A"]} => state_id![Start],
                transition_id!{["A"]->["B"]} => state_id!["A"],
                transition_id!{["B"]->["C"]} => state_id!["B"],
                transition_id!{["B"]->["D"]} => state_id!["B"],
                transition_id!{["B","a"]->["B","b"]} => state_id!["B","a"]
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["A"]} => state_id!["A"],
                transition_id!{["A"]->["B"]} => state_id!["B"],
                transition_id!{["B"]->["C"]} => state_id!["C"],
                transition_id!{["B"]->["D"]} => state_id!["D"],
                transition_id!{["B","a"]->["B","b"]} => state_id!["B","b"]
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}
