use crate::Aux;
use maplit::btreemap;
use pretty_assertions::assert_eq;
use std::collections::{BTreeMap, VecDeque};
use zero_cost_state_machine_puml::frame;
use zero_cost_state_machine_puml::frames;
use zero_cost_state_machine_puml::Frame;
use zero_cost_state_machine_puml::Frames;
use zero_cost_state_machine_puml::StateId;
use zero_cost_state_machine_puml::TransitionId;
use zero_cost_state_machine_puml::{state_id, transition_id};

fn keys_by_reference<K, V>(m: &BTreeMap<K, V>) -> BTreeMap<&K, V>
where
    V: Clone,
    K: Ord,
{
    let mut o = BTreeMap::new();
    for (k, v) in m {
        o.insert(k, v.clone());
    }
    o
}

fn all_by_reference<K, V>(m: &BTreeMap<K, V>) -> BTreeMap<&K, &V>
where
    K: Ord,
{
    let mut o = BTreeMap::new();
    for (k, v) in m {
        o.insert(k, v);
    }
    o
}

#[test]
fn exit_composite_state() -> anyhow::Result<()> {
    let contents = r#"@startuml
            State1 -> State2: a
            state State2 {
                [*] --> process: b
                process --> [*]: c
            }
            State2 -> [*]: d
        @enduml"#;
    let (_, diagram) = zero_cost_state_machine_puml::human_readable_error(
        zero_cost_state_machine_puml::plantuml,
    )(contents)?;
    let aux = Aux::new(&diagram)?;
    let transition_to_start_redirection = &btreemap! {
        transition_id!{["State1"]->["State2"]:"a"} => state_id!["State2",Start]
    };
    let transition_from_end_redirection = &btreemap! {
        transition_id!{["State2"]->[End]:"d"} => state_id!["State2",End]
    };
    let child_node_canonical_name = &btreemap! {
        state_id![End] => "End".into(),
        state_id!["State1"] => "State1".into(),
        state_id!["State2"] => "State2".into(),
        state_id!["State2", Start] => "Start".into(),
        state_id!["State2", End] => "End".into(),
        state_id!["State2", "process"] => "Process".into(),
    };
    let edge_canonical_name = &btreemap! {
        transition_id!{["State1"]->["State2"]:"a"} => Some("A".into()),
        transition_id!{["State2"]->[End]:"d"} => Some("D".into()),
        transition_id!{["State2",Start]->["State2","process"]:"b"} => Some("B".into()),
        transition_id!{["State2","process"]->["State2",End]:"c"} => Some("C".into()),
    };
    let relative_canonical_name = &btreemap! {
        transition_id!{["State1"]->["State2"]:"a"} => (
            vec!["node".into(), "State1".into()],
            1,
            vec!["state2".into(), "node".into(), "Start".into()],
            2
        ),
        transition_id!{["State2"]->[End]:"d"} => (
            vec!["node".into(), "End".into()],
            2,
            vec!["super".into(), "node".into(), "End".into()],
            1
        ),
        transition_id!{["State2",Start]->["State2","process"]:"b"} => (
            vec!["node".into(), "Start".into()],
            2,
            vec!["node".into(), "Process".into()],
            2
        ),
        transition_id!{["State2","process"]->["State2",End]:"c"} => (
            vec!["node".into(), "Process".into()],
            2,
            vec!["node".into(), "End".into()],
            2
        )
    };
    assert_eq!(
        Aux {
            transition_to_start_redirection: all_by_reference(transition_to_start_redirection),
            transition_from_end_redirection: all_by_reference(transition_from_end_redirection),
            child_node_canonical_name: keys_by_reference(child_node_canonical_name),
            edge_canonical_name: keys_by_reference(edge_canonical_name),
            relative_canonical_name: keys_by_reference(relative_canonical_name),
        },
        aux
    );
    Ok(())
}
//
// #[test]
// fn ex() -> anyhow::Result<()> {
//     let contents = r#"@startuml
//             scale 600 width
//
//             [*] -> State1
//             State1 --> State2 : Succeeded
//             State1 --> [*] : Aborted
//             State2 --> State3 : Succeeded
//             State2 --> [*] : Aborted
//             state State3 {
//                 state "Accumulate Enough Data\nLong State Name" as long1
//                 long1 : Just a test
//                 [*] --> long1
//                 long1 --> long1 : New Data
//                 long1 --> ProcessData : Enough Data
//                 long1 --> [*]
//             }
//             State3 --> State3 : Failed
//             State3 --> [*] : Succeeded / Save Result
//             State3 --> [*] : Aborted
//
//             @enduml"#;
//     let (_, diagram) =
//         zero_cost_state_machine_puml::human_readable_error(zero_cost_state_machine_puml::plantuml)(contents)?;
//     let aux = Aux::new(&diagram)?;
//     let child_node_canonical_name = &btreemap! {
//
//     };
//     let edge_canonical_name = &btreemap! {
//
//     };
//     let relative_canonical_name = &btreemap! {
//
//     };
//     assert_eq!(
//         Aux {
//             transition_to_start_redirection: Default::default(),
//             transition_from_end_redirection: Default::default(),
//             child_node_canonical_name: keys_by_reference(child_node_canonical_name),
//             edge_canonical_name: keys_by_reference(edge_canonical_name),
//             relative_canonical_name: keys_by_reference(relative_canonical_name)
//         },
//         aux
//     );
//     Ok(())
// }
