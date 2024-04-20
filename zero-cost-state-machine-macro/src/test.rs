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

#[test]
fn fill_in_void() -> anyhow::Result<()> {
    let contents = r#"@startuml
        scale 600 width
        
        [*] -> State1
        state State2 {
           [*] -> State3
           state State3 {
               [*] -> go
           }
        }
        state State1 {
           [*] -> ss1
           ss1 -> State2.State3.go
        }
        @enduml"#;
    let (_, diagram) = zero_cost_state_machine_puml::human_readable_error(
        zero_cost_state_machine_puml::plantuml,
    )(contents)?;
    let aux = Aux::new(&diagram)?;
    let transition_to_start_redirection = &btreemap! {
        transition_id!{[Start]->["State1"]} => state_id!["State1",Start],
        transition_id!{["State2",Start]->["State2","State3"]} => state_id!["State2","State3",Start],
    };
    let transition_from_end_redirection = &btreemap! {};
    let child_node_canonical_name = &btreemap! {
        state_id![Start] => "Start".into(),
        state_id!["State1"] => "State1".into(),
        state_id!["State1",Start] => "Start".into(),
        state_id!["State1","ss1"] => "Ss1".into(),
        state_id!["State2"] => "State2".into(),
        state_id!["State2", Start] => "Start".into(),
        state_id!["State2","State3"] => "State3".into(),
        state_id!["State2","State3",Start] => "Start".into(),
        state_id!["State2","State3","go"] => "Go".into(),
    };
    let edge_canonical_name = &btreemap! {
        transition_id!{[Start]->["State1"]} => None,
        transition_id!{["State1",Start]->["State1","ss1"]} => None,
        transition_id!{["State1","ss1"]->["State2","State3","go"]} => None,
        transition_id!{["State2",Start]->["State2","State3"]} => None,
        transition_id!{["State2","State3",Start]->["State2","State3","go"]} => None,
    };
    let relative_canonical_name = &btreemap! {
        transition_id!{[Start]->["State1"]} => (
            vec!["node".into(), "Start".into()],
            1,
            vec!["state1".into(), "node".into(), "Start".into()],
            2
        ),
        transition_id!{["State1",Start]->["State1","ss1"]} => (
            vec!["node".into(), "Start".into()],
            2,
            vec!["node".into(), "Ss1".into()],
            2
        ),
        transition_id!{["State1","ss1"]->["State2","State3","go"]} => (
            vec!["node".into(), "Ss1".into()],
            2,
            vec!["super".into(), "state2".into(), "state3".into(), "node".into(), "Go".into()],
            3
        ),
        transition_id!{["State2",Start]->["State2","State3"]} => (
            vec!["node".into(), "Start".into()],
            2,
            vec!["state3".into(), "node".into(), "Start".into()],
            3
        ),
        transition_id!{["State2","State3",Start]->["State2","State3","go"]} => (
            vec!["node".into(), "Start".into()],
            3,
            vec!["node".into(), "Go".into()],
            3
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
