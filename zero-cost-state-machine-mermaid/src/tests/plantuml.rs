use internal::{frame, frames};
use internal::{state_id, transition_id};
use maplit::{btreemap, btreeset};
use crate::{Frame, Frames, StateId, TransitionId, Diagram, StateStereoType};
use std::collections::VecDeque;
use pretty_assertions::assert_eq;

use crate::{human_readable_error, mermaid};
#[test]
fn empty() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert!(diagram.is_empty());
    Ok(())
}

#[test]
fn newline() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert!(diagram.is_empty());
    Ok(())
}

#[test]
fn state() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state state1 
            state state2
            @enduml
        "#;
    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id!["state1"] => state_id![],
                state_id!["state2"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["state1"],
                    state_id!["state2"],
                },
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn unknown_stereotype() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state state1 <<Warning>>
            @enduml
        "#;
    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_stereotype: btreemap! {
                state_id!["state1"] => StateStereoType::Other("Warning".to_string()),
            },
            state_parent: btreemap! {
                state_id!["state1"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["state1"],
                },
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn transition() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            State1 --> State2
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id!["State1"] => state_id![],
                state_id!["State2"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["State1"],
                    state_id!["State2"]
                }
            },
            state_transition_out: btreemap! {
                state_id!["State1"] => btreeset!{transition_id!{["State1"]->["State2"]},},
            },
            state_transition_in: btreemap! {
                state_id!["State2"] => btreeset!{transition_id!{["State1"]->["State2"]},},
            },
            transition_from: btreemap! {
                transition_id!{["State1"] -> ["State2"]} => state_id!["State1"]
            },
            transition_to: btreemap! {
                transition_id!{["State1"] -> ["State2"]} => state_id!["State2"]
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn transition_from_start() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            [*] --> State1
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id!["State1"] => state_id![],
                state_id![Start] => state_id![]
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["State1"],
                    state_id![Start]
                }
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset!{transition_id!{[Start] -> ["State1"]},},
            },
            state_transition_in: btreemap! {
                state_id!["State1"] => btreeset!{transition_id!{[Start] -> ["State1"]},},
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["State1"]} => state_id![Start]
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["State1"]} => state_id!["State1"]
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn transition_to_end() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            
            State1 --> [*]
            
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id![End] => state_id![],
                state_id!["State1"] => state_id![]
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![End],
                    state_id!["State1"]
                }
            },
            state_transition_out: btreemap! {
                state_id!["State1"] => btreeset!{transition_id!{["State1"]->[End]},},
            },
            state_transition_in: btreemap! {
                state_id![End] => btreeset!{transition_id!{["State1"]->[End]},},
            },
            transition_from: btreemap! {
                transition_id!{["State1"]->[End]} => state_id!["State1"]
            },
            transition_to: btreemap! {
                transition_id!{["State1"]->[End]} => state_id![End]
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn simple_state() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            
            [*] --> State1
            State1 --> [*]
            State1 : this is a string
            State1 : this is another string
            
            State1 -> State2
            State2 --> [*]
            
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_description: btreemap! {
                state_id!("State1") => vec![
                    "this is a string".to_string(),
                    "this is another string".to_string()
                ]
            },
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id![End] => state_id![],
                state_id!["State1"] => state_id![],
                state_id!["State2"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id![End],
                    state_id!["State1"],
                    state_id!["State2"]
                }
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset!{transition_id!{[Start]->["State1"]},},
                state_id!["State1"] => btreeset!{
                    transition_id!{["State1"]->[End]},
                    transition_id!{["State1"]->["State2"]},
                },
                state_id!["State2"] => btreeset!{
                    transition_id!{["State2"]->[End]},
                },
            },
            state_transition_in: btreemap! {
                state_id![End] => btreeset! {
                    transition_id!(["State1"]->[End]),
                    transition_id!(["State2"]->[End])
                },
                state_id!["State1"] => btreeset! {
                    transition_id!([Start]->["State1"])
                },
                state_id!["State2"] => btreeset! {
                    transition_id!(["State1"]->["State2"])
                }
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["State1"]} => state_id![Start],
                transition_id!{["State1"]->[End]} => state_id!["State1"],
                transition_id!{["State1"]->["State2"]} => state_id!["State1"],
                transition_id!{["State2"]->[End]} => state_id!["State2"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["State1"]} => state_id!["State1"],
                transition_id!{["State1"]->[End]} => state_id![End],
                transition_id!{["State1"]->["State2"]} => state_id!["State2"],
                transition_id!{["State2"]->[End]} => state_id![End],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn change_state_rendering() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            hide empty description
            [*] --> State1
            State1 --> [*]
            State1 : this is a string
            State1 : this is another string
            
            State1 -> State2
            State2 --> [*]
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_description: btreemap! {
                state_id!("State1") => vec![
                    "this is a string".to_string(),
                    "this is another string".to_string()
                ]
            },
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id![End] => state_id![],
                state_id!["State1"] => state_id![],
                state_id!["State2"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id![End],
                    state_id!["State1"],
                    state_id!["State2"]
                }
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset!{transition_id!{[Start]->["State1"]},},
                state_id!["State1"] => btreeset!{
                    transition_id!{["State1"]->[End]},
                    transition_id!{["State1"]->["State2"]},
                },
                state_id!["State2"] => btreeset!{
                    transition_id!{["State2"]->[End]},
                },
            },
            state_transition_in: btreemap! {
                state_id![End] => btreeset! {
                    transition_id!(["State1"]->[End]),
                    transition_id!(["State2"]->[End])
                },
                state_id!["State1"] => btreeset! {
                    transition_id!([Start]->["State1"])
                },
                state_id!["State2"] => btreeset! {
                    transition_id!(["State1"]->["State2"])
                }
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["State1"]} => state_id![Start],
                transition_id!{["State1"]->[End]} => state_id!["State1"],
                transition_id!{["State1"]->["State2"]} => state_id!["State1"],
                transition_id!{["State2"]->[End]} => state_id!["State2"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["State1"]} => state_id!["State1"],
                transition_id!{["State1"]->[End]} => state_id![End],
                transition_id!{["State1"]->["State2"]} => state_id!["State2"],
                transition_id!{["State2"]->[End]} => state_id![End],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn link_description() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            Configuring --> Idle: EvConfig
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id!["Configuring"] => state_id![],
                state_id!["Idle"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["Configuring"],
                    state_id!["Idle"]
                }
            },
            state_transition_out: btreemap! {
                state_id!["Configuring"] => btreeset!{
                    transition_id!{["Configuring"]->["Idle"]:"EvConfig"},
                },
            },
            state_transition_in: btreemap! {
                state_id!["Idle"] => btreeset! {
                    transition_id!(["Configuring"]->["Idle"]:"EvConfig")
                }
            },
            transition_from: btreemap! {
                transition_id!{["Configuring"]->["Idle"]:"EvConfig"} => state_id!["Configuring"],
            },
            transition_to: btreemap! {
                transition_id!{["Configuring"]->["Idle"]:"EvConfig"} => state_id!["Idle"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn empty_composite_state() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state State {
            }
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id!["State"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["State"]
                }
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn transition_to_end_in_composite_state() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state State2 {
                process --> [*]: a
            }
            State2 -> [*]: b
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id![End] => state_id![],
                state_id!["State2"] => state_id![],
                state_id!["State2",End] => state_id!["State2"],
                state_id!["State2","process"] => state_id!["State2"],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![End],
                    state_id!["State2"]
                },
                state_id!["State2"] => btreeset! {
                    state_id!["State2",End],
                    state_id!["State2","process"]
                },
            },
            state_transition_out: btreemap! {
                state_id!["State2"] => btreeset! {
                    transition_id!{["State2"]->[End]:"b"},
                },
                state_id!["State2","process"] => btreeset! {
                    transition_id!{["State2","process"]->["State2",End]:"a"},
                },
            },
            state_transition_in: btreemap! {
                state_id![End] => btreeset! {
                    transition_id!{["State2"]->[End]:"b"},
                },
                state_id!["State2",End] => btreeset! {
                    transition_id!{["State2","process"]->["State2",End]:"a"},
                },
            },
            transition_from: btreemap! {
                transition_id!{["State2"]->[End]:"b"} => state_id!["State2"],
                transition_id!{["State2","process"]->["State2",End]:"a"} => state_id!["State2","process"],
            },
            transition_to: btreemap! {
                transition_id!{["State2"]->[End]:"b"} => state_id![End],
                transition_id!{["State2","process"]->["State2",End]:"a"} => state_id!["State2",End],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn quoted_composite_state() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state "Not Shooting State" as NotShooting {
            }
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_alias: btreemap! {
                state_id!["NotShooting"] => "Not Shooting State".to_string(),
            },
            state_parent: btreemap! {
                state_id!["NotShooting"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["NotShooting"],
                },
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn quoted_composite_state2() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state NotShooting as "Not Shooting State" {
            }
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_alias: btreemap! {
                state_id!["NotShooting"] => "Not Shooting State".to_string(),
            },
            state_parent: btreemap! {
                state_id!["NotShooting"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["NotShooting"],
                },
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn transition_to_history_in_composite() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state State {
                A -> [H]: Comment
            }
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id!["State"] => state_id![],
                state_id!["State","A"] => state_id!["State"],
                state_id!["State",History] => state_id!["State"],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["State"]
                },
                state_id!["State"] => btreeset! {
                    state_id!["State","A"],
                    state_id!["State",History]
                }
            },
            state_transition_out: btreemap! {
                state_id!["State","A"] => btreeset! {
                    transition_id!{["State","A"]->["State",History]:"Comment"},
                },
            },
            state_transition_in: btreemap! {
                state_id!["State",History] => btreeset! {
                    transition_id!{["State","A"]->["State",History]:"Comment"},
                },
            },
            transition_from: btreemap! {
                transition_id!{["State","A"]->["State",History]:"Comment"} => state_id!["State","A"],
            },
            transition_to: btreemap! {
                transition_id!{["State","A"]->["State",History]:"Comment"} => state_id!["State",History],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn nested_composite_state() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            scale 350 width
            [*] --> NotShooting
            
            state NotShooting {
              [*] --> Idle
              Idle --> Configuring : EvConfig
              Configuring --> Idle : EvConfig
            }
            
            state Configuring {
              [*] --> NewValueSelection
              NewValueSelection --> NewValuePreview : EvNewValue
              NewValuePreview --> NewValueSelection : EvNewValueRejected
              NewValuePreview --> NewValueSelection : EvNewValueSaved
            
              state NewValuePreview {
                 State1 -> State2
              }
            
            }
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id!["NotShooting"] =>  state_id![],
                state_id!["NotShooting",Start] =>  state_id!["NotShooting"],
                state_id!["NotShooting","Configuring"] =>  state_id!["NotShooting"],
                state_id!["NotShooting","Configuring",Start] =>  state_id!["NotShooting","Configuring"],
                state_id!["NotShooting","Configuring","NewValuePreview"] =>  state_id!["NotShooting","Configuring"],
                state_id!["NotShooting","Configuring","NewValuePreview","State1"] =>  state_id!["NotShooting","Configuring","NewValuePreview"],
                state_id!["NotShooting","Configuring","NewValuePreview","State2"] =>  state_id!["NotShooting","Configuring","NewValuePreview"],
                state_id!["NotShooting","Configuring","NewValueSelection"] => state_id!["NotShooting","Configuring"],
                state_id!["NotShooting","Idle"] => state_id!["NotShooting"]
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id!["NotShooting"],
                },
                state_id!["NotShooting"] => btreeset! {
                    state_id!["NotShooting",Start],
                    state_id!["NotShooting","Configuring"],
                    state_id!["NotShooting","Idle"]
                },
                state_id!["NotShooting","Configuring"] => btreeset! {
                    state_id!["NotShooting","Configuring",Start],
                    state_id!["NotShooting","Configuring","NewValuePreview"],
                    state_id!["NotShooting","Configuring","NewValueSelection"]
                },
                state_id!["NotShooting","Configuring","NewValuePreview"] => btreeset! {
                    state_id!["NotShooting","Configuring","NewValuePreview","State1"],
                    state_id!["NotShooting","Configuring","NewValuePreview","State2"],
                },
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["NotShooting"]},
                },
                state_id!["NotShooting",Start] => btreeset! {
                    transition_id!{["NotShooting",Start]->["NotShooting","Idle"]},
                },
                state_id!["NotShooting","Configuring"] => btreeset! {
                    transition_id!{["NotShooting","Configuring"]->["NotShooting","Idle"]:"EvConfig"},
                },
                state_id!["NotShooting","Configuring",Start] => btreeset! {
                    transition_id!{["NotShooting","Configuring",Start]->["NotShooting","Configuring","NewValueSelection"]},
                },
                state_id!["NotShooting","Configuring","NewValuePreview"] => btreeset! {
                    transition_id!{["NotShooting","Configuring","NewValuePreview"]->["NotShooting","Configuring","NewValueSelection"]:"EvNewValueRejected"},
                    transition_id!{["NotShooting","Configuring","NewValuePreview"]->["NotShooting","Configuring","NewValueSelection"]:"EvNewValueSaved"},
                },
                state_id!["NotShooting","Configuring","NewValuePreview","State1"] => btreeset! {
                    transition_id!{["NotShooting","Configuring","NewValuePreview","State1"]->["NotShooting","Configuring","NewValuePreview","State2"]},
                },
                state_id!["NotShooting","Configuring","NewValueSelection"] => btreeset! {
                    transition_id!{["NotShooting","Configuring","NewValueSelection"]->["NotShooting","Configuring","NewValuePreview"]:"EvNewValue"},
                },
                state_id!["NotShooting","Idle"] => btreeset! {
                    transition_id!{["NotShooting","Idle"]->["NotShooting","Configuring"]:"EvConfig"},
                },
            },
            state_transition_in: btreemap! {
                state_id!["NotShooting"] => btreeset! {
                    transition_id!{[Start]->["NotShooting"]},
                },
                state_id!["NotShooting","Configuring"] => btreeset! {
                    transition_id!{["NotShooting","Idle"]->["NotShooting","Configuring"]:"EvConfig"},
                },
                state_id!["NotShooting","Configuring","NewValuePreview"] => btreeset! {
                    transition_id!{["NotShooting","Configuring","NewValueSelection"]->["NotShooting","Configuring","NewValuePreview"]:"EvNewValue"},
                },
                state_id!["NotShooting","Configuring","NewValuePreview","State2"] => btreeset! {
                    transition_id!{["NotShooting","Configuring","NewValuePreview","State1"]->["NotShooting","Configuring","NewValuePreview","State2"]},
                },
                state_id!["NotShooting","Configuring","NewValueSelection"] => btreeset! {
                    transition_id!{["NotShooting","Configuring",Start]->["NotShooting","Configuring","NewValueSelection"]},
                    transition_id!{["NotShooting","Configuring","NewValuePreview"]->["NotShooting","Configuring","NewValueSelection"]:"EvNewValueRejected"},
                    transition_id!{["NotShooting","Configuring","NewValuePreview"]->["NotShooting","Configuring","NewValueSelection"]:"EvNewValueSaved"},
                },
                state_id!["NotShooting","Idle"] => btreeset! {
                    transition_id!{["NotShooting",Start]->["NotShooting","Idle"]},
                    transition_id!{["NotShooting","Configuring"]->["NotShooting","Idle"]:"EvConfig"},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["NotShooting"]} => state_id![Start],
                transition_id!{["NotShooting",Start]->["NotShooting","Idle"]} => state_id!["NotShooting",Start],
                transition_id!{["NotShooting","Configuring"]->["NotShooting","Idle"]:"EvConfig"} => state_id!["NotShooting","Configuring"],
                transition_id!{["NotShooting","Configuring",Start]->["NotShooting","Configuring","NewValueSelection"]} => state_id!["NotShooting","Configuring",Start],
                transition_id!{["NotShooting","Configuring","NewValuePreview"]->["NotShooting","Configuring","NewValueSelection"]:"EvNewValueRejected"} => state_id!["NotShooting","Configuring","NewValuePreview"],
                transition_id!{["NotShooting","Configuring","NewValuePreview"]->["NotShooting","Configuring","NewValueSelection"]:"EvNewValueSaved"} => state_id!["NotShooting","Configuring","NewValuePreview"],
                transition_id!{["NotShooting","Configuring","NewValuePreview","State1"]->["NotShooting","Configuring","NewValuePreview","State2"]} => state_id!["NotShooting","Configuring","NewValuePreview","State1"],
                transition_id!{["NotShooting","Configuring","NewValueSelection"]->["NotShooting","Configuring","NewValuePreview"]:"EvNewValue"} => state_id!["NotShooting","Configuring","NewValueSelection"],
                transition_id!{["NotShooting","Idle"]->["NotShooting","Configuring"]:"EvConfig"} => state_id!["NotShooting","Idle"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["NotShooting"]} => state_id!["NotShooting"],
                transition_id!{["NotShooting",Start]->["NotShooting","Idle"]} => state_id!["NotShooting","Idle"],
                transition_id!{["NotShooting","Configuring"]->["NotShooting","Idle"]:"EvConfig"} => state_id!["NotShooting","Idle"],
                transition_id!{["NotShooting","Configuring",Start]->["NotShooting","Configuring","NewValueSelection"]} => state_id!["NotShooting","Configuring","NewValueSelection"],
                transition_id!{["NotShooting","Configuring","NewValuePreview"]->["NotShooting","Configuring","NewValueSelection"]:"EvNewValueRejected"} => state_id!["NotShooting","Configuring","NewValueSelection"],
                transition_id!{["NotShooting","Configuring","NewValuePreview"]->["NotShooting","Configuring","NewValueSelection"]:"EvNewValueSaved"} => state_id!["NotShooting","Configuring","NewValueSelection"],
                transition_id!{["NotShooting","Configuring","NewValuePreview","State1"]->["NotShooting","Configuring","NewValuePreview","State2"]} => state_id!["NotShooting","Configuring","NewValuePreview","State2"],
                transition_id!{["NotShooting","Configuring","NewValueSelection"]->["NotShooting","Configuring","NewValuePreview"]:"EvNewValue"} => state_id!["NotShooting","Configuring","NewValuePreview"],
                transition_id!{["NotShooting","Idle"]->["NotShooting","Configuring"]:"EvConfig"} => state_id!["NotShooting","Configuring"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn sub_state() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state A {
                state X {
                }
            }
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id!["A"] => state_id![],
                state_id!["A","X"] => state_id!["A"]
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["A"]
                },
                state_id!["A"] => btreeset! {
                    state_id!["A","X"]
                }
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn sub_state_to_sub_state() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state A {
              state X {
              }
              state Y {
              }
            }
             
            state B {
              state Z {
              }
            }
            
            X --> Z
            Z --> Y
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id!["A"] => state_id![],
                state_id!["A","X"] => state_id!["A"],
                state_id!["A","Y"] => state_id!["A"],
                state_id!["B"] => state_id![],
                state_id!["B","Z"] => state_id!["B"],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["A"],
                    state_id!["B"],
                },
                state_id!["A"] => btreeset! {
                    state_id!["A","X"],
                    state_id!["A","Y"],
                },
                state_id!["B"] => btreeset! {
                    state_id!["B","Z"],
                },
            },
            state_transition_out: btreemap! {
                state_id!["A","X"] => btreeset! {
                    transition_id!{["A","X"]->["B","Z"]},
                },
                state_id!["B","Z"] => btreeset! {
                    transition_id!{["B","Z"]->["A","Y"]},
                },
            },
            state_transition_in: btreemap! {
                state_id!["A","Y"] => btreeset! {
                    transition_id!{["B","Z"]->["A","Y"]},
                },
                state_id!["B","Z"] => btreeset! {
                    transition_id!{["A","X"]->["B","Z"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{["A","X"]->["B","Z"]} => state_id!["A","X"],
                transition_id!{["B","Z"]->["A","Y"]} => state_id!["B","Z"],
            },
            transition_to: btreemap! {
                transition_id!{["A","X"]->["B","Z"]} => state_id!["B","Z"],
                transition_id!{["B","Z"]->["A","Y"]} => state_id!["A","Y"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn long_name() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            scale 600 width
            
            [*] -> State1
            State1 --> State2 : Succeeded
            State1 --> [*] : Aborted
            State2 --> State3 : Succeeded
            State2 --> [*] : Aborted
            state State3 {
              state "Accumulate Enough Data\nLong State Name" as long1
              long1 : Just a test
              [*] --> long1
              long1 --> long1 : New Data
              long1 --> ProcessData : Enough Data
            }
            State3 --> State3 : Failed
            State3 --> [*] : Succeeded / Save Result
            State3 --> [*] : Aborted
            
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_alias: btreemap! {
                state_id!["State3","long1"] => "Accumulate Enough Data\nLong State Name".to_string(),
            },
            state_description: btreemap! {
                state_id!["State3","long1"] => vec![
                    "Just a test".to_string(),
                ],
            },
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id![End] => state_id![],
                state_id!["State1"] => state_id![],
                state_id!["State2"] => state_id![],
                state_id!["State3"] => state_id![],
                state_id!["State3",Start] => state_id!["State3"],
                state_id!["State3","ProcessData"] => state_id!["State3"],
                state_id!["State3","long1"] => state_id!["State3"],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id![End],
                    state_id!["State1"],
                    state_id!["State2"],
                    state_id!["State3"],
                },
                state_id!["State3"] => btreeset! {
                    state_id!["State3",Start],
                    state_id!["State3","ProcessData"],
                    state_id!["State3","long1"],
                },
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["State1"]},
                },
                state_id!["State1"] => btreeset! {
                    transition_id!{["State1"]->[End]:"Aborted"},
                    transition_id!{["State1"]->["State2"]:"Succeeded"},
                },
                state_id!["State2"] => btreeset! {
                    transition_id!{["State2"]->[End]:"Aborted"},
                    transition_id!{["State2"]->["State3"]:"Succeeded"},
                },
                state_id!["State3"] => btreeset! {
                    transition_id!{["State3"]->[End]:"Aborted"},
                    transition_id!{["State3"]->[End]:"Succeeded / Save Result"},
                    transition_id!{["State3"]->["State3"]:"Failed"},
                },
                state_id!["State3",Start] => btreeset! {
                    transition_id!{["State3",Start]->["State3","long1"]},
                },
                state_id!["State3","long1"] => btreeset! {
                    transition_id!{["State3","long1"]->["State3","ProcessData"]:"Enough Data"},
                    transition_id!{["State3","long1"]->["State3","long1"]:"New Data"},
                },
            },
            state_transition_in: btreemap! {
                state_id![End] => btreeset! {
                    transition_id!{["State1"]->[End]:"Aborted"},
                    transition_id!{["State2"]->[End]:"Aborted"},
                    transition_id!{["State3"]->[End]:"Aborted"},
                    transition_id!{["State3"]->[End]:"Succeeded / Save Result"},
                },
                state_id!["State1"] => btreeset! {
                    transition_id!{[Start]->["State1"]},
                },
                state_id!["State2"] => btreeset! {
                    transition_id!{["State1"]->["State2"]:"Succeeded"},
                },
                state_id!["State3"] => btreeset! {
                    transition_id!{["State2"]->["State3"]:"Succeeded"},
                    transition_id!{["State3"]->["State3"]:"Failed"},
                },
                state_id!["State3","ProcessData"] => btreeset! {
                    transition_id!{["State3","long1"]->["State3","ProcessData"]:"Enough Data"},
                },
                state_id!["State3","long1"] => btreeset! {
                    transition_id!{["State3",Start]->["State3","long1"]},
                    transition_id!{["State3","long1"]->["State3","long1"]:"New Data"},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["State1"]} => state_id![Start],
                transition_id!{["State1"]->[End]:"Aborted"} => state_id!["State1"],
                transition_id!{["State1"]->["State2"]:"Succeeded"} => state_id!["State1"],
                transition_id!{["State2"]->[End]:"Aborted"} => state_id!["State2"],
                transition_id!{["State2"]->["State3"]:"Succeeded"} => state_id!["State2"],
                transition_id!{["State3"]->[End]:"Aborted"} => state_id!["State3"],
                transition_id!{["State3"]->[End]:"Succeeded / Save Result"} => state_id!["State3"],
                transition_id!{["State3"]->["State3"]:"Failed"} => state_id!["State3"],
                transition_id!{["State3",Start]->["State3","long1"]} => state_id!["State3",Start],
                transition_id!{["State3","long1"]->["State3","ProcessData"]:"Enough Data"} => state_id!["State3","long1"],
                transition_id!{["State3","long1"]->["State3","long1"]:"New Data"} => state_id!["State3","long1"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["State1"]} => state_id!["State1"],
                transition_id!{["State1"]->[End]:"Aborted"} => state_id![End],
                transition_id!{["State1"]->["State2"]:"Succeeded"} => state_id!["State2"],
                transition_id!{["State2"]->[End]:"Aborted"} => state_id![End],
                transition_id!{["State2"]->["State3"]:"Succeeded"} => state_id!["State3"],
                transition_id!{["State3"]->[End]:"Aborted"} => state_id![End],
                transition_id!{["State3"]->[End]:"Succeeded / Save Result"} => state_id![End],
                transition_id!{["State3"]->["State3"]:"Failed"} => state_id!["State3"],
                transition_id!{["State3",Start]->["State3","long1"]} => state_id!["State3","long1"],
                transition_id!{["State3","long1"]->["State3","ProcessData"]:"Enough Data"} => state_id!["State3","ProcessData"],
                transition_id!{["State3","long1"]->["State3","long1"]:"New Data"} => state_id!["State3","long1"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn history() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            [*] -> State1
            State1 --> State2 : Succeeded
            State1 --> [*] : Aborted
            State2 --> State3 : Succeeded
            State2 --> [*] : Aborted
            state State3 {
              state "Accumulate Enough Data" as long1
              long1 : Just a test
              [*] --> long1
              long1 --> long1 : New Data
              long1 --> ProcessData : Enough Data
              State2 --> [H]: Resume
            }
            State3 --> State2 : Pause
            State2 --> State3[H*]: DeepResume
            State3 --> State3 : Failed
            State3 --> [*] : Succeeded / Save Result
            State3 --> [*] : Aborted
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_alias: btreemap! {
                state_id!["State3","long1"] => "Accumulate Enough Data".to_string(),
            },
            state_description: btreemap! {
                state_id!["State3","long1"] => vec![
                    "Just a test".to_string(),
                ],
            },
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id![End] => state_id![],
                state_id!["State1"] => state_id![],
                state_id!["State2"] => state_id![],
                state_id!["State3"] => state_id![],
                state_id!["State3",Start] => state_id!["State3"],
                state_id!["State3",History] => state_id!["State3"],
                state_id!["State3",DeepHistory] => state_id!["State3"],
                state_id!["State3","ProcessData"] => state_id!["State3"],
                state_id!["State3","long1"] => state_id!["State3"],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id![End],
                    state_id!["State1"],
                    state_id!["State2"],
                    state_id!["State3"],
                },
                state_id!["State3"] => btreeset! {
                    state_id!["State3",Start],
                    state_id!["State3",History],
                    state_id!["State3",DeepHistory],
                    state_id!["State3","ProcessData"],
                    state_id!["State3","long1"],
                },
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["State1"]},
                },
                state_id!["State1"] => btreeset! {
                    transition_id!{["State1"]->[End]:"Aborted"},
                    transition_id!{["State1"]->["State2"]:"Succeeded"},
                },
                state_id!["State2"] => btreeset! {
                    transition_id!{["State2"]->[End]:"Aborted"},
                    transition_id!{["State2"]->["State3"]:"Succeeded"},
                    transition_id!{["State2"]->["State3",History]:"Resume"},
                    transition_id!{["State2"]->["State3",DeepHistory]:"DeepResume"},
                },
                state_id!["State3"] => btreeset! {
                    transition_id!{["State3"]->[End]:"Aborted"},
                    transition_id!{["State3"]->[End]:"Succeeded / Save Result"},
                    transition_id!{["State3"]->["State2"]:"Pause"},
                    transition_id!{["State3"]->["State3"]:"Failed"},
                },
                state_id!["State3",Start] => btreeset! {
                    transition_id!{["State3",Start]->["State3","long1"]},
                },
                state_id!["State3","long1"] => btreeset! {
                    transition_id!{["State3","long1"]->["State3","ProcessData"]:"Enough Data"},
                    transition_id!{["State3","long1"]->["State3","long1"]:"New Data"},
                },
            },
            state_transition_in: btreemap! {
                state_id![End] => btreeset! {
                    transition_id!{["State1"]->[End]:"Aborted"},
                    transition_id!{["State2"]->[End]:"Aborted"},
                    transition_id!{["State3"]->[End]:"Aborted"},
                    transition_id!{["State3"]->[End]:"Succeeded / Save Result"},
                },
                state_id!["State1"] => btreeset! {
                    transition_id!{[Start]->["State1"]},
                },
                state_id!["State2"] => btreeset! {
                    transition_id!{["State1"]->["State2"]:"Succeeded"},
                    transition_id!{["State3"]->["State2"]:"Pause"},
                },
                state_id!["State3"] => btreeset! {
                    transition_id!{["State2"]->["State3"]:"Succeeded"},
                    transition_id!{["State3"]->["State3"]:"Failed"},
                },
                state_id!["State3",History] => btreeset! {
                    transition_id!{["State2"]->["State3",History]:"Resume"},
                },
                state_id!["State3",DeepHistory] => btreeset! {
                    transition_id!{["State2"]->["State3",DeepHistory]:"DeepResume"},
                },
                state_id!["State3","ProcessData"] => btreeset! {
                    transition_id!{["State3","long1"]->["State3","ProcessData"]:"Enough Data"},
                },
                state_id!["State3","long1"] => btreeset! {
                    transition_id!{["State3",Start]->["State3","long1"]},
                    transition_id!{["State3","long1"]->["State3","long1"]:"New Data"},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["State1"]} => state_id![Start],
                transition_id!{["State1"]->[End]:"Aborted"} => state_id!["State1"],
                transition_id!{["State1"]->["State2"]:"Succeeded"} => state_id!["State1"],
                transition_id!{["State2"]->[End]:"Aborted"} => state_id!["State2"],
                transition_id!{["State2"]->["State3"]:"Succeeded"} => state_id!["State2"],
                transition_id!{["State2"]->["State3",History]:"Resume"} => state_id!["State2"],
                transition_id!{["State2"]->["State3",DeepHistory]:"DeepResume"} => state_id!["State2"],
                transition_id!{["State3"]->[End]:"Aborted"} => state_id!["State3"],
                transition_id!{["State3"]->[End]:"Succeeded / Save Result"} => state_id!["State3"],
                transition_id!{["State3"]->["State2"]:"Pause"} => state_id!["State3"],
                transition_id!{["State3"]->["State3"]:"Failed"} => state_id!["State3"],
                transition_id!{["State3",Start]->["State3","long1"]} => state_id!["State3",Start],
                transition_id!{["State3","long1"]->["State3","ProcessData"]:"Enough Data"} => state_id!["State3","long1"],
                transition_id!{["State3","long1"]->["State3","long1"]:"New Data"} => state_id!["State3","long1"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["State1"]} => state_id!["State1"],
                transition_id!{["State1"]->[End]:"Aborted"} => state_id![End],
                transition_id!{["State1"]->["State2"]:"Succeeded"} => state_id!["State2"],
                transition_id!{["State2"]->[End]:"Aborted"} => state_id![End],
                transition_id!{["State2"]->["State3"]:"Succeeded"} => state_id!["State3"],
                transition_id!{["State2"]->["State3",History]:"Resume"} => state_id!["State3",History],
                transition_id!{["State2"]->["State3",DeepHistory]:"DeepResume"} => state_id!["State3",DeepHistory],
                transition_id!{["State3"]->[End]:"Aborted"} => state_id![End],
                transition_id!{["State3"]->[End]:"Succeeded / Save Result"} => state_id![End],
                transition_id!{["State3"]->["State2"]:"Pause"} => state_id!["State2"],
                transition_id!{["State3"]->["State3"]:"Failed"} => state_id!["State3"],
                transition_id!{["State3",Start]->["State3","long1"]} => state_id!["State3","long1"],
                transition_id!{["State3","long1"]->["State3","ProcessData"]:"Enough Data"} => state_id!["State3","ProcessData"],
                transition_id!{["State3","long1"]->["State3","long1"]:"New Data"} => state_id!["State3","long1"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn fork() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            
            state fork_state <<fork>>
            [*] --> fork_state
            fork_state --> State2
            fork_state --> State3
            
            state join_state <<join>>
            State2 --> join_state
            State3 --> join_state
            join_state --> State4
            State4 --> [*]
            
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id![End] => state_id![],
                state_id!["State2"] => state_id![],
                state_id!["State3"] => state_id![],
                state_id!["State4"] => state_id![],
                state_id!["fork_state"] => state_id![],
                state_id!["join_state"] => state_id![],
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
                },
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["fork_state"]},
                },
                state_id!["State2"] => btreeset! {
                    transition_id!{["State2"]->["join_state"]},
                },
                state_id!["State3"] => btreeset! {
                    transition_id!{["State3"]->["join_state"]},
                },
                state_id!["State4"] => btreeset! {
                    transition_id!{["State4"]->[End]},
                },
                state_id!["fork_state"] => btreeset! {
                    transition_id!{["fork_state"]->["State2"]},
                    transition_id!{["fork_state"]->["State3"]},
                },
                state_id!["join_state"] => btreeset! {
                    transition_id!{["join_state"]->["State4"]},
                },
            },
            state_transition_in: btreemap! {
                state_id![End] => btreeset! {
                    transition_id!{["State4"]->[End]},
                },
                state_id!["State2"] => btreeset! {
                    transition_id!{["fork_state"]->["State2"]},
                },
                state_id!["State3"] => btreeset! {
                    transition_id!{["fork_state"]->["State3"]},
                },
                state_id!["State4"] => btreeset! {
                    transition_id!{["join_state"]->["State4"]},
                },
                state_id!["fork_state"] => btreeset! {
                    transition_id!{[Start]->["fork_state"]},
                },
                state_id!["join_state"] => btreeset! {
                    transition_id!{["State2"]->["join_state"]},
                    transition_id!{["State3"]->["join_state"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["fork_state"]} => state_id![Start],
                transition_id!{["State2"]->["join_state"]} => state_id!["State2"],
                transition_id!{["State3"]->["join_state"]} => state_id!["State3"],
                transition_id!{["State4"]->[End]} => state_id!["State4"],
                transition_id!{["fork_state"]->["State2"]} => state_id!["fork_state"],
                transition_id!{["fork_state"]->["State3"]} => state_id!["fork_state"],
                transition_id!{["join_state"]->["State4"]} => state_id!["join_state"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["fork_state"]} => state_id!["fork_state"],
                transition_id!{["State2"]->["join_state"]} => state_id!["join_state"],
                transition_id!{["State3"]->["join_state"]} => state_id!["join_state"],
                transition_id!{["State4"]->[End]} => state_id![End],
                transition_id!{["fork_state"]->["State2"]} => state_id!["State2"],
                transition_id!{["fork_state"]->["State3"]} => state_id!["State3"],
                transition_id!{["join_state"]->["State4"]} => state_id!["State4"],
            },
            state_stereotype: btreemap! {
                state_id!["fork_state"] => StateStereoType::Fork,
                state_id!["join_state"] => StateStereoType::Join,
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn concurrent_horizontal() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            [*] --> Active
            
            state Active {
              [*] -> NumLockOff
              NumLockOff --> NumLockOn : EvNumLockPressed
              NumLockOn --> NumLockOff : EvNumLockPressed
              --
              [*] -> CapsLockOff
              CapsLockOff --> CapsLockOn : EvCapsLockPressed
              CapsLockOn --> CapsLockOff : EvCapsLockPressed
              --
              [*] -> ScrollLockOff
              ScrollLockOff --> ScrollLockOn : EvScrollLockPressed
              ScrollLockOn --> ScrollLockOff : EvScrollLockPressed
            }
            
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id!["Active"] => state_id![],
                state_id!["Active",Start] => state_id!["Active"],
                state_id!["Active","CapsLockOff"] => state_id!["Active"],
                state_id!["Active","CapsLockOn"] => state_id!["Active"],
                state_id!["Active","NumLockOff"] => state_id!["Active"],
                state_id!["Active","NumLockOn"] => state_id!["Active"],
                state_id!["Active","ScrollLockOff"] => state_id!["Active"],
                state_id!["Active","ScrollLockOn"] => state_id!["Active"],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id!["Active"],
                },
                state_id!["Active"] => btreeset! {
                    state_id!["Active",Start],
                    state_id!["Active","CapsLockOff"],
                    state_id!["Active","CapsLockOn"],
                    state_id!["Active","NumLockOff"],
                    state_id!["Active","NumLockOn"],
                    state_id!["Active","ScrollLockOff"],
                    state_id!["Active","ScrollLockOn"],
                },
            },
            state_children_are_concurrent: btreeset! {
                state_id!["Active"]
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["Active"]},
                },
                state_id!["Active",Start] => btreeset! {
                    transition_id!{["Active",Start]->["Active","CapsLockOff"]},
                    transition_id!{["Active",Start]->["Active","NumLockOff"]},
                    transition_id!{["Active",Start]->["Active","ScrollLockOff"]},
                },
                state_id!["Active","CapsLockOff"] => btreeset! {
                    transition_id!{["Active","CapsLockOff"]->["Active","CapsLockOn"]:"EvCapsLockPressed"},
                },
                state_id!["Active","CapsLockOn"] => btreeset! {
                    transition_id!{["Active","CapsLockOn"]->["Active","CapsLockOff"]:"EvCapsLockPressed"},
                },
                state_id!["Active","NumLockOff"] => btreeset! {
                    transition_id!{["Active","NumLockOff"]->["Active","NumLockOn"]:"EvNumLockPressed"},
                },
                state_id!["Active","NumLockOn"] => btreeset! {
                    transition_id!{["Active","NumLockOn"]->["Active","NumLockOff"]:"EvNumLockPressed"},
                },
                state_id!["Active","ScrollLockOff"] => btreeset! {
                    transition_id!{["Active","ScrollLockOff"]->["Active","ScrollLockOn"]:"EvScrollLockPressed"},
                },
                state_id!["Active","ScrollLockOn"] => btreeset! {
                    transition_id!{["Active","ScrollLockOn"]->["Active","ScrollLockOff"]:"EvScrollLockPressed"},
                },
            },
            state_transition_in: btreemap! {
                state_id!["Active"] => btreeset! {
                    transition_id!{[Start]->["Active"]},
                },
                state_id!["Active","CapsLockOff"] => btreeset! {
                    transition_id!{["Active",Start]->["Active","CapsLockOff"]},
                    transition_id!{["Active","CapsLockOn"]->["Active","CapsLockOff"]:"EvCapsLockPressed"},
                },
                state_id!["Active","CapsLockOn"] => btreeset! {
                    transition_id!{["Active","CapsLockOff"]->["Active","CapsLockOn"]:"EvCapsLockPressed"},
                },
                state_id!["Active","NumLockOff"] => btreeset! {
                    transition_id!{["Active",Start]->["Active","NumLockOff"]},
                    transition_id!{["Active","NumLockOn"]->["Active","NumLockOff"]:"EvNumLockPressed"},
                },
                state_id!["Active","NumLockOn"] => btreeset! {
                    transition_id!{["Active","NumLockOff"]->["Active","NumLockOn"]:"EvNumLockPressed"},
                },
                state_id!["Active","ScrollLockOff"] => btreeset! {
                    transition_id!{["Active",Start]->["Active","ScrollLockOff"]},
                    transition_id!{["Active","ScrollLockOn"]->["Active","ScrollLockOff"]:"EvScrollLockPressed"},
                },
                state_id!["Active","ScrollLockOn"] => btreeset! {
                    transition_id!{["Active","ScrollLockOff"]->["Active","ScrollLockOn"]:"EvScrollLockPressed"},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["Active"]} => state_id![Start],
                transition_id!{["Active",Start]->["Active","CapsLockOff"]} => state_id!["Active",Start],
                transition_id!{["Active",Start]->["Active","NumLockOff"]} => state_id!["Active",Start],
                transition_id!{["Active",Start]->["Active","ScrollLockOff"]} => state_id!["Active",Start],
                transition_id!{["Active","CapsLockOff"]->["Active","CapsLockOn"]:"EvCapsLockPressed"} => state_id!["Active","CapsLockOff"],
                transition_id!{["Active","CapsLockOn"]->["Active","CapsLockOff"]:"EvCapsLockPressed"} => state_id!["Active","CapsLockOn"],
                transition_id!{["Active","NumLockOff"]->["Active","NumLockOn"]:"EvNumLockPressed"} => state_id!["Active","NumLockOff"],
                transition_id!{["Active","NumLockOn"]->["Active","NumLockOff"]:"EvNumLockPressed"} => state_id!["Active","NumLockOn"],
                transition_id!{["Active","ScrollLockOff"]->["Active","ScrollLockOn"]:"EvScrollLockPressed"} => state_id!["Active","ScrollLockOff"],
                transition_id!{["Active","ScrollLockOn"]->["Active","ScrollLockOff"]:"EvScrollLockPressed"} => state_id!["Active","ScrollLockOn"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["Active"]} => state_id!["Active"],
                transition_id!{["Active",Start]->["Active","CapsLockOff"]} => state_id!["Active","CapsLockOff"],
                transition_id!{["Active",Start]->["Active","NumLockOff"]} => state_id!["Active","NumLockOff"],
                transition_id!{["Active",Start]->["Active","ScrollLockOff"]} => state_id!["Active","ScrollLockOff"],
                transition_id!{["Active","CapsLockOff"]->["Active","CapsLockOn"]:"EvCapsLockPressed"} => state_id!["Active","CapsLockOn"],
                transition_id!{["Active","CapsLockOn"]->["Active","CapsLockOff"]:"EvCapsLockPressed"} => state_id!["Active","CapsLockOff"],
                transition_id!{["Active","NumLockOff"]->["Active","NumLockOn"]:"EvNumLockPressed"} => state_id!["Active","NumLockOn"],
                transition_id!{["Active","NumLockOn"]->["Active","NumLockOff"]:"EvNumLockPressed"} => state_id!["Active","NumLockOff"],
                transition_id!{["Active","ScrollLockOff"]->["Active","ScrollLockOn"]:"EvScrollLockPressed"} => state_id!["Active","ScrollLockOn"],
                transition_id!{["Active","ScrollLockOn"]->["Active","ScrollLockOff"]:"EvScrollLockPressed"} => state_id!["Active","ScrollLockOff"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn concurrent_vertical() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            [*] --> Active
            
            state Active {
              [*] -> NumLockOff
              NumLockOff --> NumLockOn : EvNumLockPressed
              NumLockOn --> NumLockOff : EvNumLockPressed
              ||
              [*] -> CapsLockOff
              CapsLockOff --> CapsLockOn : EvCapsLockPressed
              CapsLockOn --> CapsLockOff : EvCapsLockPressed
              ||
              [*] -> ScrollLockOff
              ScrollLockOff --> ScrollLockOn : EvScrollLockPressed
              ScrollLockOn --> ScrollLockOff : EvScrollLockPressed
            }
            
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id!["Active"] => state_id![],
                state_id!["Active",Start] => state_id!["Active"],
                state_id!["Active","CapsLockOff"] => state_id!["Active"],
                state_id!["Active","CapsLockOn"] => state_id!["Active"],
                state_id!["Active","NumLockOff"] => state_id!["Active"],
                state_id!["Active","NumLockOn"] => state_id!["Active"],
                state_id!["Active","ScrollLockOff"] => state_id!["Active"],
                state_id!["Active","ScrollLockOn"] => state_id!["Active"],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id!["Active"],
                },
                state_id!["Active"] => btreeset! {
                    state_id!["Active",Start],
                    state_id!["Active","CapsLockOff"],
                    state_id!["Active","CapsLockOn"],
                    state_id!["Active","NumLockOff"],
                    state_id!["Active","NumLockOn"],
                    state_id!["Active","ScrollLockOff"],
                    state_id!["Active","ScrollLockOn"],
                },
            },
            state_children_are_concurrent: btreeset! {
                state_id!["Active"]
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["Active"]},
                },
                state_id!["Active",Start] => btreeset! {
                    transition_id!{["Active",Start]->["Active","CapsLockOff"]},
                    transition_id!{["Active",Start]->["Active","NumLockOff"]},
                    transition_id!{["Active",Start]->["Active","ScrollLockOff"]},
                },
                state_id!["Active","CapsLockOff"] => btreeset! {
                    transition_id!{["Active","CapsLockOff"]->["Active","CapsLockOn"]:"EvCapsLockPressed"},
                },
                state_id!["Active","CapsLockOn"] => btreeset! {
                    transition_id!{["Active","CapsLockOn"]->["Active","CapsLockOff"]:"EvCapsLockPressed"},
                },
                state_id!["Active","NumLockOff"] => btreeset! {
                    transition_id!{["Active","NumLockOff"]->["Active","NumLockOn"]:"EvNumLockPressed"},
                },
                state_id!["Active","NumLockOn"] => btreeset! {
                    transition_id!{["Active","NumLockOn"]->["Active","NumLockOff"]:"EvNumLockPressed"},
                },
                state_id!["Active","ScrollLockOff"] => btreeset! {
                    transition_id!{["Active","ScrollLockOff"]->["Active","ScrollLockOn"]:"EvScrollLockPressed"},
                },
                state_id!["Active","ScrollLockOn"] => btreeset! {
                    transition_id!{["Active","ScrollLockOn"]->["Active","ScrollLockOff"]:"EvScrollLockPressed"},
                },
            },
            state_transition_in: btreemap! {
                state_id!["Active"] => btreeset! {
                    transition_id!{[Start]->["Active"]},
                },
                state_id!["Active","CapsLockOff"] => btreeset! {
                    transition_id!{["Active",Start]->["Active","CapsLockOff"]},
                    transition_id!{["Active","CapsLockOn"]->["Active","CapsLockOff"]:"EvCapsLockPressed"},
                },
                state_id!["Active","CapsLockOn"] => btreeset! {
                    transition_id!{["Active","CapsLockOff"]->["Active","CapsLockOn"]:"EvCapsLockPressed"},
                },
                state_id!["Active","NumLockOff"] => btreeset! {
                    transition_id!{["Active",Start]->["Active","NumLockOff"]},
                    transition_id!{["Active","NumLockOn"]->["Active","NumLockOff"]:"EvNumLockPressed"},
                },
                state_id!["Active","NumLockOn"] => btreeset! {
                    transition_id!{["Active","NumLockOff"]->["Active","NumLockOn"]:"EvNumLockPressed"},
                },
                state_id!["Active","ScrollLockOff"] => btreeset! {
                    transition_id!{["Active",Start]->["Active","ScrollLockOff"]},
                    transition_id!{["Active","ScrollLockOn"]->["Active","ScrollLockOff"]:"EvScrollLockPressed"},
                },
                state_id!["Active","ScrollLockOn"] => btreeset! {
                    transition_id!{["Active","ScrollLockOff"]->["Active","ScrollLockOn"]:"EvScrollLockPressed"},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["Active"]} => state_id![Start],
                transition_id!{["Active",Start]->["Active","CapsLockOff"]} => state_id!["Active",Start],
                transition_id!{["Active",Start]->["Active","NumLockOff"]} => state_id!["Active",Start],
                transition_id!{["Active",Start]->["Active","ScrollLockOff"]} => state_id!["Active",Start],
                transition_id!{["Active","CapsLockOff"]->["Active","CapsLockOn"]:"EvCapsLockPressed"} => state_id!["Active","CapsLockOff"],
                transition_id!{["Active","CapsLockOn"]->["Active","CapsLockOff"]:"EvCapsLockPressed"} => state_id!["Active","CapsLockOn"],
                transition_id!{["Active","NumLockOff"]->["Active","NumLockOn"]:"EvNumLockPressed"} => state_id!["Active","NumLockOff"],
                transition_id!{["Active","NumLockOn"]->["Active","NumLockOff"]:"EvNumLockPressed"} => state_id!["Active","NumLockOn"],
                transition_id!{["Active","ScrollLockOff"]->["Active","ScrollLockOn"]:"EvScrollLockPressed"} => state_id!["Active","ScrollLockOff"],
                transition_id!{["Active","ScrollLockOn"]->["Active","ScrollLockOff"]:"EvScrollLockPressed"} => state_id!["Active","ScrollLockOn"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["Active"]} => state_id!["Active"],
                transition_id!{["Active",Start]->["Active","CapsLockOff"]} => state_id!["Active","CapsLockOff"],
                transition_id!{["Active",Start]->["Active","NumLockOff"]} => state_id!["Active","NumLockOff"],
                transition_id!{["Active",Start]->["Active","ScrollLockOff"]} => state_id!["Active","ScrollLockOff"],
                transition_id!{["Active","CapsLockOff"]->["Active","CapsLockOn"]:"EvCapsLockPressed"} => state_id!["Active","CapsLockOn"],
                transition_id!{["Active","CapsLockOn"]->["Active","CapsLockOff"]:"EvCapsLockPressed"} => state_id!["Active","CapsLockOff"],
                transition_id!{["Active","NumLockOff"]->["Active","NumLockOn"]:"EvNumLockPressed"} => state_id!["Active","NumLockOn"],
                transition_id!{["Active","NumLockOn"]->["Active","NumLockOff"]:"EvNumLockPressed"} => state_id!["Active","NumLockOff"],
                transition_id!{["Active","ScrollLockOff"]->["Active","ScrollLockOn"]:"EvScrollLockPressed"} => state_id!["Active","ScrollLockOn"],
                transition_id!{["Active","ScrollLockOn"]->["Active","ScrollLockOff"]:"EvScrollLockPressed"} => state_id!["Active","ScrollLockOff"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn conditional_choice() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state "Req(Id)" as ReqId <<sdlreceive>>
            state "Minor(Id)" as MinorId
            state "Major(Id)" as MajorId
             
            state c <<choice>>
             
            Idle --> ReqId
            ReqId --> c
            c --> MinorId : [Id <= 10]
            c --> MajorId : [Id > 10]
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_alias: btreemap! {
                state_id!["MajorId"] => "Major(Id)".to_string(),
                state_id!["MinorId"] => "Minor(Id)".to_string(),
                state_id!["ReqId"] => "Req(Id)".to_string(),
            },
            state_stereotype: btreemap! {
                state_id!["c"] => StateStereoType::Choice,
            },
            state_parent: btreemap! {
                state_id!["Idle"] => state_id![],
                state_id!["MajorId"] => state_id![],
                state_id!["MinorId"] => state_id![],
                state_id!["ReqId"] => state_id![],
                state_id!["c"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["Idle"],
                    state_id!["MajorId"],
                    state_id!["MinorId"],
                    state_id!["ReqId"],
                    state_id!["c"],
                },
            },
            state_transition_out: btreemap! {
                state_id!["Idle"] => btreeset! {
                    transition_id!{["Idle"]->["ReqId"]},
                },
                state_id!["ReqId"] => btreeset! {
                    transition_id!{["ReqId"]->["c"]},
                },
                state_id!["c"] => btreeset! {
                    transition_id!{["c"]->["MajorId"]:"[Id > 10]"},
                    transition_id!{["c"]->["MinorId"]:"[Id <= 10]"},
                },
            },
            state_transition_in: btreemap! {
                state_id!["MajorId"] => btreeset! {
                    transition_id!{["c"]->["MajorId"]:"[Id > 10]"},
                },
                state_id!["MinorId"] => btreeset! {
                    transition_id!{["c"]->["MinorId"]:"[Id <= 10]"},
                },
                state_id!["ReqId"] => btreeset! {
                    transition_id!{["Idle"]->["ReqId"]},
                },
                state_id!["c"] => btreeset! {
                    transition_id!{["ReqId"]->["c"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{["Idle"]->["ReqId"]} => state_id!["Idle"],
                transition_id!{["ReqId"]->["c"]} => state_id!["ReqId"],
                transition_id!{["c"]->["MajorId"]:"[Id > 10]"} => state_id!["c"],
                transition_id!{["c"]->["MinorId"]:"[Id <= 10]"} => state_id!["c"],
            },
            transition_to: btreemap! {
                transition_id!{["Idle"]->["ReqId"]} => state_id!["ReqId"],
                transition_id!{["ReqId"]->["c"]} => state_id!["c"],
                transition_id!{["c"]->["MajorId"]:"[Id > 10]"} => state_id!["MajorId"],
                transition_id!{["c"]->["MinorId"]:"[Id <= 10]"} => state_id!["MinorId"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn stereotypes_full_example() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state start1  <<start>>
            state choice1 <<choice>>
            state fork1   <<fork>>
            state join2   <<join>>
            state end3    <<end>>
            
            [*]     --> choice1 : from start\nto choice
            start1  --> choice1 : from start stereo\nto choice
            
            choice1 --> fork1   : from choice\nto fork
            choice1 --> join2   : from choice\nto join
            choice1 --> end3    : from choice\nto end stereo
            
            fork1   ---> State1 : from fork\nto state
            fork1   --> State2  : from fork\nto state
            
            State2  --> join2   : from state\nto join
            State1  --> [*]     : from state\nto end
            
            join2   --> [*]     : from join\nto end
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_stereotype: btreemap! {
                state_id!["choice1"] => StateStereoType::Choice,
                state_id!["end3"] => StateStereoType::Other(String::from("end")),
                state_id!["fork1"] => StateStereoType::Fork,
                state_id!["join2"] => StateStereoType::Join,
                state_id!["start1"] => StateStereoType::Other(String::from("start")),
            },
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id![End] => state_id![],
                state_id!["State1"] => state_id![],
                state_id!["State2"] => state_id![],
                state_id!["choice1"] => state_id![],
                state_id!["end3"] => state_id![],
                state_id!["fork1"] => state_id![],
                state_id!["join2"] => state_id![],
                state_id!["start1"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id![End],
                    state_id!["State1"],
                    state_id!["State2"],
                    state_id!["choice1"],
                    state_id!["end3"],
                    state_id!["fork1"],
                    state_id!["join2"],
                    state_id!["start1"],
                },
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["choice1"]:"from start\\nto choice"},
                },
                state_id!["State1"] => btreeset! {
                    transition_id!{["State1"]->[End]:"from state\\nto end"},
                },
                state_id!["State2"] => btreeset! {
                    transition_id!{["State2"]->["join2"]:"from state\\nto join"},
                },
                state_id!["choice1"] => btreeset! {
                    transition_id!{["choice1"]->["end3"]:"from choice\\nto end stereo"},
                    transition_id!{["choice1"]->["fork1"]:"from choice\\nto fork"},
                    transition_id!{["choice1"]->["join2"]:"from choice\\nto join"},
                },
                state_id!["fork1"] => btreeset! {
                    transition_id!{["fork1"]->["State1"]:"from fork\\nto state"},
                    transition_id!{["fork1"]->["State2"]:"from fork\\nto state"},
                },
                state_id!["join2"] => btreeset! {
                    transition_id!{["join2"]->[End]:"from join\\nto end"},
                },
                state_id!["start1"] => btreeset! {
                    transition_id!{["start1"]->["choice1"]:"from start stereo\\nto choice"},
                },
            },
            state_transition_in: btreemap! {
                state_id![End] => btreeset! {
                    transition_id!{["State1"]->[End]:"from state\\nto end"},
                    transition_id!{["join2"]->[End]:"from join\\nto end"},
                },
                state_id!["State1"] => btreeset! {
                    transition_id!{["fork1"]->["State1"]:"from fork\\nto state"},
                },
                state_id!["State2"] => btreeset! {
                    transition_id!{["fork1"]->["State2"]:"from fork\\nto state"},
                },
                state_id!["choice1"] => btreeset! {
                    transition_id!{[Start]->["choice1"]:"from start\\nto choice"},
                    transition_id!{["start1"]->["choice1"]:"from start stereo\\nto choice"},
                },
                state_id!["end3"] => btreeset! {
                    transition_id!{["choice1"]->["end3"]:"from choice\\nto end stereo"},
                },
                state_id!["fork1"] => btreeset! {
                    transition_id!{["choice1"]->["fork1"]:"from choice\\nto fork"},
                },
                state_id!["join2"] => btreeset! {
                    transition_id!{["State2"]->["join2"]:"from state\\nto join"},
                    transition_id!{["choice1"]->["join2"]:"from choice\\nto join"},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["choice1"]:"from start\\nto choice"} => state_id![Start],
                transition_id!{["State1"]->[End]:"from state\\nto end"} => state_id!["State1"],
                transition_id!{["State2"]->["join2"]:"from state\\nto join"} => state_id!["State2"],
                transition_id!{["choice1"]->["end3"]:"from choice\\nto end stereo"} => state_id!["choice1"],
                transition_id!{["choice1"]->["fork1"]:"from choice\\nto fork"} => state_id!["choice1"],
                transition_id!{["choice1"]->["join2"]:"from choice\\nto join"} => state_id!["choice1"],
                transition_id!{["fork1"]->["State1"]:"from fork\\nto state"} => state_id!["fork1"],
                transition_id!{["fork1"]->["State2"]:"from fork\\nto state"} => state_id!["fork1"],
                transition_id!{["join2"]->[End]:"from join\\nto end"} => state_id!["join2"],
                transition_id!{["start1"]->["choice1"]:"from start stereo\\nto choice"} => state_id!["start1"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["choice1"]:"from start\\nto choice"} => state_id!["choice1"],
                transition_id!{["State1"]->[End]:"from state\\nto end"} => state_id![End],
                transition_id!{["State2"]->["join2"]:"from state\\nto join"} => state_id!["join2"],
                transition_id!{["choice1"]->["end3"]:"from choice\\nto end stereo"} => state_id!["end3"],
                transition_id!{["choice1"]->["fork1"]:"from choice\\nto fork"} => state_id!["fork1"],
                transition_id!{["choice1"]->["join2"]:"from choice\\nto join"} => state_id!["join2"],
                transition_id!{["fork1"]->["State1"]:"from fork\\nto state"} => state_id!["State1"],
                transition_id!{["fork1"]->["State2"]:"from fork\\nto state"} => state_id!["State2"],
                transition_id!{["join2"]->[End]:"from join\\nto end"} => state_id![End],
                transition_id!{["start1"]->["choice1"]:"from start stereo\\nto choice"} => state_id!["choice1"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn point() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state Somp {
              state entry1 <<entryPoint>>
              state entry2 <<entryPoint>>
              state sin
              entry1 --> sin
              entry2 -> sin
              sin -> sin2
              sin2 --> exitA <<exitPoint>>
            }
            
            [*] --> entry1
            exitA --> Foo
            Foo1 -> entry2
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_stereotype: btreemap! {
                state_id!["Somp","entry1"] => StateStereoType::Other(String::from("entryPoint")),
                state_id!["Somp","entry2"] => StateStereoType::Other(String::from("entryPoint")),
                state_id!["Somp","exitA"] => StateStereoType::Other(String::from("exitPoint")),
            },
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id!["Foo"] => state_id![],
                state_id!["Foo1"] => state_id![],
                state_id!["Somp"] => state_id![],
                state_id!["Somp","entry1"] => state_id!["Somp"],
                state_id!["Somp","entry2"] => state_id!["Somp"],
                state_id!["Somp","exitA"] => state_id!["Somp"],
                state_id!["Somp","sin"] => state_id!["Somp"],
                state_id!["Somp","sin2"] => state_id!["Somp"],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id!["Foo"],
                    state_id!["Foo1"],
                    state_id!["Somp"],
                },
                state_id!["Somp"] => btreeset! {
                    state_id!["Somp","entry1"],
                    state_id!["Somp","entry2"],
                    state_id!["Somp","exitA"],
                    state_id!["Somp","sin"],
                    state_id!["Somp","sin2"],
                },
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["Somp","entry1"]},
                },
                state_id!["Foo1"] => btreeset! {
                    transition_id!{["Foo1"]->["Somp","entry2"]},
                },
                state_id!["Somp","entry1"] => btreeset! {
                    transition_id!{["Somp","entry1"]->["Somp","sin"]},
                },
                state_id!["Somp","entry2"] => btreeset! {
                    transition_id!{["Somp","entry2"]->["Somp","sin"]},
                },
                state_id!["Somp","exitA"] => btreeset! {
                    transition_id!{["Somp","exitA"]->["Foo"]},
                },
                state_id!["Somp","sin"] => btreeset! {
                    transition_id!{["Somp","sin"]->["Somp","sin2"]},
                },
                state_id!["Somp","sin2"] => btreeset! {
                    transition_id!{["Somp","sin2"]->["Somp","exitA"]},
                },
            },
            state_transition_in: btreemap! {
                state_id!["Foo"] => btreeset! {
                    transition_id!{["Somp","exitA"]->["Foo"]},
                },
                state_id!["Somp","entry1"] => btreeset! {
                    transition_id!{[Start]->["Somp","entry1"]},
                },
                state_id!["Somp","entry2"] => btreeset! {
                    transition_id!{["Foo1"]->["Somp","entry2"]},
                },
                state_id!["Somp","exitA"] => btreeset! {
                    transition_id!{["Somp","sin2"]->["Somp","exitA"]},
                },
                state_id!["Somp","sin"] => btreeset! {
                    transition_id!{["Somp","entry1"]->["Somp","sin"]},
                    transition_id!{["Somp","entry2"]->["Somp","sin"]},
                },
                state_id!["Somp","sin2"] => btreeset! {
                    transition_id!{["Somp","sin"]->["Somp","sin2"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["Somp","entry1"]} => state_id![Start],
                transition_id!{["Foo1"]->["Somp","entry2"]} => state_id!["Foo1"],
                transition_id!{["Somp","entry1"]->["Somp","sin"]} => state_id!["Somp","entry1"],
                transition_id!{["Somp","entry2"]->["Somp","sin"]} => state_id!["Somp","entry2"],
                transition_id!{["Somp","exitA"]->["Foo"]} => state_id!["Somp","exitA"],
                transition_id!{["Somp","sin"]->["Somp","sin2"]} => state_id!["Somp","sin"],
                transition_id!{["Somp","sin2"]->["Somp","exitA"]} => state_id!["Somp","sin2"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["Somp","entry1"]} => state_id!["Somp","entry1"],
                transition_id!{["Foo1"]->["Somp","entry2"]} => state_id!["Somp","entry2"],
                transition_id!{["Somp","entry1"]->["Somp","sin"]} => state_id!["Somp","sin"],
                transition_id!{["Somp","entry2"]->["Somp","sin"]} => state_id!["Somp","sin"],
                transition_id!{["Somp","exitA"]->["Foo"]} => state_id!["Foo"],
                transition_id!{["Somp","sin"]->["Somp","sin2"]} => state_id!["Somp","sin2"],
                transition_id!{["Somp","sin2"]->["Somp","exitA"]} => state_id!["Somp","exitA"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn pin() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state Somp {
              state entry1 <<inputPin>>
              state entry2 <<inputPin>>
              state sin
              entry1 --> sin
              entry2 -> sin
              sin -> sin2
              sin2 --> exitA <<outputPin>>
            }
            
            [*] --> entry1
            exitA --> Foo
            Foo1 -> entry2
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_stereotype: btreemap! {
                state_id!["Somp","entry1"] => StateStereoType::Other(String::from("inputPin")),
                state_id!["Somp","entry2"] => StateStereoType::Other(String::from("inputPin")),
                state_id!["Somp","exitA"] => StateStereoType::Other(String::from("outputPin")),
            },
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id!["Foo"] => state_id![],
                state_id!["Foo1"] => state_id![],
                state_id!["Somp"] => state_id![],
                state_id!["Somp","entry1"] => state_id!["Somp"],
                state_id!["Somp","entry2"] => state_id!["Somp"],
                state_id!["Somp","exitA"] => state_id!["Somp"],
                state_id!["Somp","sin"] => state_id!["Somp"],
                state_id!["Somp","sin2"] => state_id!["Somp"],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id!["Foo"],
                    state_id!["Foo1"],
                    state_id!["Somp"],
                },
                state_id!["Somp"] => btreeset! {
                    state_id!["Somp","entry1"],
                    state_id!["Somp","entry2"],
                    state_id!["Somp","exitA"],
                    state_id!["Somp","sin"],
                    state_id!["Somp","sin2"],
                },
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["Somp","entry1"]},
                },
                state_id!["Foo1"] => btreeset! {
                    transition_id!{["Foo1"]->["Somp","entry2"]},
                },
                state_id!["Somp","entry1"] => btreeset! {
                    transition_id!{["Somp","entry1"]->["Somp","sin"]},
                },
                state_id!["Somp","entry2"] => btreeset! {
                    transition_id!{["Somp","entry2"]->["Somp","sin"]},
                },
                state_id!["Somp","exitA"] => btreeset! {
                    transition_id!{["Somp","exitA"]->["Foo"]},
                },
                state_id!["Somp","sin"] => btreeset! {
                    transition_id!{["Somp","sin"]->["Somp","sin2"]},
                },
                state_id!["Somp","sin2"] => btreeset! {
                    transition_id!{["Somp","sin2"]->["Somp","exitA"]},
                },
            },
            state_transition_in: btreemap! {
                state_id!["Foo"] => btreeset! {
                    transition_id!{["Somp","exitA"]->["Foo"]},
                },
                state_id!["Somp","entry1"] => btreeset! {
                    transition_id!{[Start]->["Somp","entry1"]},
                },
                state_id!["Somp","entry2"] => btreeset! {
                    transition_id!{["Foo1"]->["Somp","entry2"]},
                },
                state_id!["Somp","exitA"] => btreeset! {
                    transition_id!{["Somp","sin2"]->["Somp","exitA"]},
                },
                state_id!["Somp","sin"] => btreeset! {
                    transition_id!{["Somp","entry1"]->["Somp","sin"]},
                    transition_id!{["Somp","entry2"]->["Somp","sin"]},
                },
                state_id!["Somp","sin2"] => btreeset! {
                    transition_id!{["Somp","sin"]->["Somp","sin2"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["Somp","entry1"]} => state_id![Start],
                transition_id!{["Foo1"]->["Somp","entry2"]} => state_id!["Foo1"],
                transition_id!{["Somp","entry1"]->["Somp","sin"]} => state_id!["Somp","entry1"],
                transition_id!{["Somp","entry2"]->["Somp","sin"]} => state_id!["Somp","entry2"],
                transition_id!{["Somp","exitA"]->["Foo"]} => state_id!["Somp","exitA"],
                transition_id!{["Somp","sin"]->["Somp","sin2"]} => state_id!["Somp","sin"],
                transition_id!{["Somp","sin2"]->["Somp","exitA"]} => state_id!["Somp","sin2"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["Somp","entry1"]} => state_id!["Somp","entry1"],
                transition_id!{["Foo1"]->["Somp","entry2"]} => state_id!["Somp","entry2"],
                transition_id!{["Somp","entry1"]->["Somp","sin"]} => state_id!["Somp","sin"],
                transition_id!{["Somp","entry2"]->["Somp","sin"]} => state_id!["Somp","sin"],
                transition_id!{["Somp","exitA"]->["Foo"]} => state_id!["Foo"],
                transition_id!{["Somp","sin"]->["Somp","sin2"]} => state_id!["Somp","sin2"],
                transition_id!{["Somp","sin2"]->["Somp","exitA"]} => state_id!["Somp","exitA"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn expansion() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state Somp {
              state entry1 <<expansionInput>>
              state entry2 <<expansionInput>>
              state sin
              entry1 --> sin
              entry2 -> sin
              sin -> sin2
              sin2 --> exitA <<expansionOutput>>
            }
            
            [*] --> entry1
            exitA --> Foo
            Foo1 -> entry2
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_stereotype: btreemap! {
                state_id!["Somp","entry1"] => StateStereoType::Other(String::from("expansionInput")),
                state_id!["Somp","entry2"] => StateStereoType::Other(String::from("expansionInput")),
                state_id!["Somp","exitA"] => StateStereoType::Other(String::from("expansionOutput")),
            },
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id!["Foo"] => state_id![],
                state_id!["Foo1"] => state_id![],
                state_id!["Somp"] => state_id![],
                state_id!["Somp","entry1"] => state_id!["Somp"],
                state_id!["Somp","entry2"] => state_id!["Somp"],
                state_id!["Somp","exitA"] => state_id!["Somp"],
                state_id!["Somp","sin"] => state_id!["Somp"],
                state_id!["Somp","sin2"] => state_id!["Somp"],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id!["Foo"],
                    state_id!["Foo1"],
                    state_id!["Somp"],
                },
                state_id!["Somp"] => btreeset! {
                    state_id!["Somp","entry1"],
                    state_id!["Somp","entry2"],
                    state_id!["Somp","exitA"],
                    state_id!["Somp","sin"],
                    state_id!["Somp","sin2"],
                },
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["Somp","entry1"]},
                },
                state_id!["Foo1"] => btreeset! {
                    transition_id!{["Foo1"]->["Somp","entry2"]},
                },
                state_id!["Somp","entry1"] => btreeset! {
                    transition_id!{["Somp","entry1"]->["Somp","sin"]},
                },
                state_id!["Somp","entry2"] => btreeset! {
                    transition_id!{["Somp","entry2"]->["Somp","sin"]},
                },
                state_id!["Somp","exitA"] => btreeset! {
                    transition_id!{["Somp","exitA"]->["Foo"]},
                },
                state_id!["Somp","sin"] => btreeset! {
                    transition_id!{["Somp","sin"]->["Somp","sin2"]},
                },
                state_id!["Somp","sin2"] => btreeset! {
                    transition_id!{["Somp","sin2"]->["Somp","exitA"]},
                },
            },
            state_transition_in: btreemap! {
                state_id!["Foo"] => btreeset! {
                    transition_id!{["Somp","exitA"]->["Foo"]},
                },
                state_id!["Somp","entry1"] => btreeset! {
                    transition_id!{[Start]->["Somp","entry1"]},
                },
                state_id!["Somp","entry2"] => btreeset! {
                    transition_id!{["Foo1"]->["Somp","entry2"]},
                },
                state_id!["Somp","exitA"] => btreeset! {
                    transition_id!{["Somp","sin2"]->["Somp","exitA"]},
                },
                state_id!["Somp","sin"] => btreeset! {
                    transition_id!{["Somp","entry1"]->["Somp","sin"]},
                    transition_id!{["Somp","entry2"]->["Somp","sin"]},
                },
                state_id!["Somp","sin2"] => btreeset! {
                    transition_id!{["Somp","sin"]->["Somp","sin2"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["Somp","entry1"]} => state_id![Start],
                transition_id!{["Foo1"]->["Somp","entry2"]} => state_id!["Foo1"],
                transition_id!{["Somp","entry1"]->["Somp","sin"]} => state_id!["Somp","entry1"],
                transition_id!{["Somp","entry2"]->["Somp","sin"]} => state_id!["Somp","entry2"],
                transition_id!{["Somp","exitA"]->["Foo"]} => state_id!["Somp","exitA"],
                transition_id!{["Somp","sin"]->["Somp","sin2"]} => state_id!["Somp","sin"],
                transition_id!{["Somp","sin2"]->["Somp","exitA"]} => state_id!["Somp","sin2"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["Somp","entry1"]} => state_id!["Somp","entry1"],
                transition_id!{["Foo1"]->["Somp","entry2"]} => state_id!["Somp","entry2"],
                transition_id!{["Somp","entry1"]->["Somp","sin"]} => state_id!["Somp","sin"],
                transition_id!{["Somp","entry2"]->["Somp","sin"]} => state_id!["Somp","sin"],
                transition_id!{["Somp","exitA"]->["Foo"]} => state_id!["Foo"],
                transition_id!{["Somp","sin"]->["Somp","sin2"]} => state_id!["Somp","sin2"],
                transition_id!{["Somp","sin2"]->["Somp","exitA"]} => state_id!["Somp","exitA"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn arrow_direction() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            
            [*] -up-> First
            First -right-> Second
            Second -down-> Third
            Third -left-> Last
            
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id!["First"] => state_id![],
                state_id!["Last"] => state_id![],
                state_id!["Second"] => state_id![],
                state_id!["Third"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id!["First"],
                    state_id!["Last"],
                    state_id!["Second"],
                    state_id!["Third"],
                },
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["First"]},
                },
                state_id!["First"] => btreeset! {
                    transition_id!{["First"]->["Second"]},
                },
                state_id!["Second"] => btreeset! {
                    transition_id!{["Second"]->["Third"]},
                },
                state_id!["Third"] => btreeset! {
                    transition_id!{["Third"]->["Last"]},
                },
            },
            state_transition_in: btreemap! {
                state_id!["First"] => btreeset! {
                    transition_id!{[Start]->["First"]},
                },
                state_id!["Last"] => btreeset! {
                    transition_id!{["Third"]->["Last"]},
                },
                state_id!["Second"] => btreeset! {
                    transition_id!{["First"]->["Second"]},
                },
                state_id!["Third"] => btreeset! {
                    transition_id!{["Second"]->["Third"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["First"]} => state_id![Start],
                transition_id!{["First"]->["Second"]} => state_id!["First"],
                transition_id!{["Second"]->["Third"]} => state_id!["Second"],
                transition_id!{["Third"]->["Last"]} => state_id!["Third"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["First"]} => state_id!["First"],
                transition_id!{["First"]->["Second"]} => state_id!["Second"],
                transition_id!{["Second"]->["Third"]} => state_id!["Third"],
                transition_id!{["Third"]->["Last"]} => state_id!["Last"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn line_color_and_style() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            State S1
            State S2
            S1 -[#DD00AA]-> S2
            S1 -left[#yellow]-> S3
            S1 -up[#red,dashed]-> S4
            S1 -right[dotted,#blue]-> S5
            
            X1 -[dashed]-> X2
            Z1 -[dotted]-> Z2
            Y1 -[#blue,bold]-> Y2
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id!["S1"] => state_id![],
                state_id!["S2"] => state_id![],
                state_id!["S3"] => state_id![],
                state_id!["S4"] => state_id![],
                state_id!["S5"] => state_id![],
                state_id!["X1"] => state_id![],
                state_id!["X2"] => state_id![],
                state_id!["Y1"] => state_id![],
                state_id!["Y2"] => state_id![],
                state_id!["Z1"] => state_id![],
                state_id!["Z2"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["S1"],
                    state_id!["S2"],
                    state_id!["S3"],
                    state_id!["S4"],
                    state_id!["S5"],
                    state_id!["X1"],
                    state_id!["X2"],
                    state_id!["Y1"],
                    state_id!["Y2"],
                    state_id!["Z1"],
                    state_id!["Z2"],
                },
            },
            state_transition_out: btreemap! {
                state_id!["S1"] => btreeset! {
                    transition_id!{["S1"]->["S2"]},
                    transition_id!{["S1"]->["S3"]},
                    transition_id!{["S1"]->["S4"]},
                    transition_id!{["S1"]->["S5"]},
                },
                state_id!["X1"] => btreeset! {
                    transition_id!{["X1"]->["X2"]},
                },
                state_id!["Y1"] => btreeset! {
                    transition_id!{["Y1"]->["Y2"]},
                },
                state_id!["Z1"] => btreeset! {
                    transition_id!{["Z1"]->["Z2"]},
                },
            },
            state_transition_in: btreemap! {
                state_id!["S2"] => btreeset! {
                    transition_id!{["S1"]->["S2"]},
                },
                state_id!["S3"] => btreeset! {
                    transition_id!{["S1"]->["S3"]},
                },
                state_id!["S4"] => btreeset! {
                    transition_id!{["S1"]->["S4"]},
                },
                state_id!["S5"] => btreeset! {
                    transition_id!{["S1"]->["S5"]},
                },
                state_id!["X2"] => btreeset! {
                    transition_id!{["X1"]->["X2"]},
                },
                state_id!["Y2"] => btreeset! {
                    transition_id!{["Y1"]->["Y2"]},
                },
                state_id!["Z2"] => btreeset! {
                    transition_id!{["Z1"]->["Z2"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{["S1"]->["S2"]} => state_id!["S1"],
                transition_id!{["S1"]->["S3"]} => state_id!["S1"],
                transition_id!{["S1"]->["S4"]} => state_id!["S1"],
                transition_id!{["S1"]->["S5"]} => state_id!["S1"],
                transition_id!{["X1"]->["X2"]} => state_id!["X1"],
                transition_id!{["Y1"]->["Y2"]} => state_id!["Y1"],
                transition_id!{["Z1"]->["Z2"]} => state_id!["Z1"],
            },
            transition_to: btreemap! {
                transition_id!{["S1"]->["S2"]} => state_id!["S2"],
                transition_id!{["S1"]->["S3"]} => state_id!["S3"],
                transition_id!{["S1"]->["S4"]} => state_id!["S4"],
                transition_id!{["S1"]->["S5"]} => state_id!["S5"],
                transition_id!{["X1"]->["X2"]} => state_id!["X2"],
                transition_id!{["Y1"]->["Y2"]} => state_id!["Y2"],
                transition_id!{["Z1"]->["Z2"]} => state_id!["Z2"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn note() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            
            [*] --> Active
            Active --> Inactive
            
            note left of Active : this is a short\nnote
            
            note right of Inactive
              A note can also
              be defined on
              several lines
            end note
            
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_note: btreemap! {
                state_id!["Active"] => vec![
                    "this is a short\\nnote".to_string(),
                ],
                state_id!["Inactive"] => vec![
                    "A note can also".to_string(),
                    "be defined on".to_string(),
                    "several lines".to_string(),
                ],
            },
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id!["Active"] => state_id![],
                state_id!["Inactive"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id!["Active"],
                    state_id!["Inactive"],
                },
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["Active"]},
                },
                state_id!["Active"] => btreeset! {
                    transition_id!{["Active"]->["Inactive"]},
                },
            },
            state_transition_in: btreemap! {
                state_id!["Active"] => btreeset! {
                    transition_id!{[Start]->["Active"]},
                },
                state_id!["Inactive"] => btreeset! {
                    transition_id!{["Active"]->["Inactive"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["Active"]} => state_id![Start],
                transition_id!{["Active"]->["Inactive"]} => state_id!["Active"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["Active"]} => state_id!["Active"],
                transition_id!{["Active"]->["Inactive"]} => state_id!["Inactive"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn floating_note() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            
            state foo
            note "This is a floating note" as N1
            
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id!["foo"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["foo"],
                },
            },
            note: vec!["This is a floating note".to_string(),],
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn note_on_link() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            [*] -> State1
            State1 --> State2
            note on link
                this is a state-transition note
            end note
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id!["State1"] => state_id![],
                state_id!["State2"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id!["State1"],
                    state_id!["State2"],
                },
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["State1"]},
                },
                state_id!["State1"] => btreeset! {
                    transition_id!{["State1"]->["State2"]},
                },
            },
            state_transition_in: btreemap! {
                state_id!["State1"] => btreeset! {
                    transition_id!{[Start]->["State1"]},
                },
                state_id!["State2"] => btreeset! {
                    transition_id!{["State1"]->["State2"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["State1"]} => state_id![Start],
                transition_id!{["State1"]->["State2"]} => state_id!["State1"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["State1"]} => state_id!["State1"],
                transition_id!{["State1"]->["State2"]} => state_id!["State2"],
            },
            transition_note: btreemap! {
                transition_id!{["State1"]->["State2"]} => vec![
                    "this is a state-transition note".to_string(),
                ],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn note_on_composite_state() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            
            [*] --> NotShooting
            
            state "Not Shooting State" as NotShooting {
              state "Idle mode" as Idle
              state "Configuring mode" as Configuring
              [*] --> Idle
              Idle --> Configuring : EvConfig
              Configuring --> Idle : EvConfig
            }
            
            note right of NotShooting : This is a note on a composite state
            
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_alias: btreemap! {
                state_id!["Configuring"] => "Configuring mode".to_string(),
                state_id!["Idle"] => "Idle mode".to_string(),
                state_id!["NotShooting"] => "Not Shooting State".to_string(),
            },
            state_note: btreemap! {
                state_id!["NotShooting"] => vec![
                    "This is a note on a composite state".to_string(),
                ],
            },
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id!["Configuring"] => state_id![],
                state_id!["Idle"] => state_id![],
                state_id!["NotShooting"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id!["Configuring"],
                    state_id!["Idle"],
                    state_id!["NotShooting"],
                },
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["Idle"]},
                    transition_id!{[Start]->["NotShooting"]},
                },
                state_id!["Configuring"] => btreeset! {
                    transition_id!{["Configuring"]->["Idle"]:"EvConfig"},
                },
                state_id!["Idle"] => btreeset! {
                    transition_id!{["Idle"]->["Configuring"]:"EvConfig"},
                },
            },
            state_transition_in: btreemap! {
                state_id!["Configuring"] => btreeset! {
                    transition_id!{["Idle"]->["Configuring"]:"EvConfig"},
                },
                state_id!["Idle"] => btreeset! {
                    transition_id!{[Start]->["Idle"]},
                    transition_id!{["Configuring"]->["Idle"]:"EvConfig"},
                },
                state_id!["NotShooting"] => btreeset! {
                    transition_id!{[Start]->["NotShooting"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["Idle"]} => state_id![Start],
                transition_id!{[Start]->["NotShooting"]} => state_id![Start],
                transition_id!{["Configuring"]->["Idle"]:"EvConfig"} => state_id!["Configuring"],
                transition_id!{["Idle"]->["Configuring"]:"EvConfig"} => state_id!["Idle"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["Idle"]} => state_id!["Idle"],
                transition_id!{[Start]->["NotShooting"]} => state_id!["NotShooting"],
                transition_id!{["Configuring"]->["Idle"]:"EvConfig"} => state_id!["Idle"],
                transition_id!{["Idle"]->["Configuring"]:"EvConfig"} => state_id!["Configuring"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn inline_color() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state Trends #FFFF77
            state Schedule #magenta
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id!["Schedule"] => state_id![],
                state_id!["Trends"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["Schedule"],
                    state_id!["Trends"],
                },
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn inline_color_composite_state() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state CurrentSite #pink {
                state HardwareSetup #lightblue {
                   state Site #brown
                    Site -[hidden]-> Controller
                    Controller -[hidden]-> Devices
                }
                state PresentationSetup{
                    Groups -[hidden]-> PlansAndGraphics
                }
                state Trends #FFFF77
                state Schedule #magenta
                state AlarmSupression
            }
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id!["CurrentSite"] => state_id![],
                state_id!["CurrentSite","AlarmSupression"] => state_id!["CurrentSite"],
                state_id!["CurrentSite","HardwareSetup"] => state_id!["CurrentSite"],
                state_id!["CurrentSite","HardwareSetup","Controller"] => state_id!["CurrentSite","HardwareSetup"],
                state_id!["CurrentSite","HardwareSetup","Devices"] => state_id!["CurrentSite","HardwareSetup"],
                state_id!["CurrentSite","HardwareSetup","Site"] => state_id!["CurrentSite","HardwareSetup"],
                state_id!["CurrentSite","PresentationSetup"] => state_id!["CurrentSite"],
                state_id!["CurrentSite","PresentationSetup","Groups"] => state_id!["CurrentSite","PresentationSetup"],
                state_id!["CurrentSite","PresentationSetup","PlansAndGraphics"] => state_id!["CurrentSite","PresentationSetup"],
                state_id!["CurrentSite","Schedule"] => state_id!["CurrentSite"],
                state_id!["CurrentSite","Trends"] => state_id!["CurrentSite"],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["CurrentSite"],
                },
                state_id!["CurrentSite"] => btreeset! {
                    state_id!["CurrentSite","AlarmSupression"],
                    state_id!["CurrentSite","HardwareSetup"],
                    state_id!["CurrentSite","PresentationSetup"],
                    state_id!["CurrentSite","Schedule"],
                    state_id!["CurrentSite","Trends"],
                },
                state_id!["CurrentSite","HardwareSetup"] => btreeset! {
                    state_id!["CurrentSite","HardwareSetup","Controller"],
                    state_id!["CurrentSite","HardwareSetup","Devices"],
                    state_id!["CurrentSite","HardwareSetup","Site"],
                },
                state_id!["CurrentSite","PresentationSetup"] => btreeset! {
                    state_id!["CurrentSite","PresentationSetup","Groups"],
                    state_id!["CurrentSite","PresentationSetup","PlansAndGraphics"],
                },
            },
            state_transition_out: btreemap! {
                state_id!["CurrentSite","HardwareSetup","Controller"] => btreeset! {
                    transition_id!{["CurrentSite","HardwareSetup","Controller"]->["CurrentSite","HardwareSetup","Devices"]},
                },
                state_id!["CurrentSite","HardwareSetup","Site"] => btreeset! {
                    transition_id!{["CurrentSite","HardwareSetup","Site"]->["CurrentSite","HardwareSetup","Controller"]},
                },
                state_id!["CurrentSite","PresentationSetup","Groups"] => btreeset! {
                    transition_id!{["CurrentSite","PresentationSetup","Groups"]->["CurrentSite","PresentationSetup","PlansAndGraphics"]},
                },
            },
            state_transition_in: btreemap! {
                state_id!["CurrentSite","HardwareSetup","Controller"] => btreeset! {
                    transition_id!{["CurrentSite","HardwareSetup","Site"]->["CurrentSite","HardwareSetup","Controller"]},
                },
                state_id!["CurrentSite","HardwareSetup","Devices"] => btreeset! {
                    transition_id!{["CurrentSite","HardwareSetup","Controller"]->["CurrentSite","HardwareSetup","Devices"]},
                },
                state_id!["CurrentSite","PresentationSetup","PlansAndGraphics"] => btreeset! {
                    transition_id!{["CurrentSite","PresentationSetup","Groups"]->["CurrentSite","PresentationSetup","PlansAndGraphics"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{["CurrentSite","HardwareSetup","Controller"]->["CurrentSite","HardwareSetup","Devices"]} => state_id!["CurrentSite","HardwareSetup","Controller"],
                transition_id!{["CurrentSite","HardwareSetup","Site"]->["CurrentSite","HardwareSetup","Controller"]} => state_id!["CurrentSite","HardwareSetup","Site"],
                transition_id!{["CurrentSite","PresentationSetup","Groups"]->["CurrentSite","PresentationSetup","PlansAndGraphics"]} => state_id!["CurrentSite","PresentationSetup","Groups"],
            },
            transition_to: btreemap! {
                transition_id!{["CurrentSite","HardwareSetup","Controller"]->["CurrentSite","HardwareSetup","Devices"]} => state_id!["CurrentSite","HardwareSetup","Devices"],
                transition_id!{["CurrentSite","HardwareSetup","Site"]->["CurrentSite","HardwareSetup","Controller"]} => state_id!["CurrentSite","HardwareSetup","Controller"],
                transition_id!{["CurrentSite","PresentationSetup","Groups"]->["CurrentSite","PresentationSetup","PlansAndGraphics"]} => state_id!["CurrentSite","PresentationSetup","PlansAndGraphics"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn skinparam() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            skinparam backgroundColor LightYellow
            skinparam state {
              StartColor MediumBlue
              EndColor Red
              BackgroundColor Peru
              BackgroundColor<<Warning>> Olive
              BorderColor Gray
              FontName Impact
            }
            
            [*] --> NotShooting
            
            state "Not Shooting State" as NotShooting {
              state "Idle mode" as Idle <<Warning>>
              state "Configuring mode" as Configuring
              [*] --> Idle
              Idle --> Configuring : EvConfig
              Configuring --> Idle : EvConfig
            }
            
            NotShooting --> [*]
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_alias: btreemap! {
                state_id!["Configuring"] => "Configuring mode".to_string(),
                state_id!["Idle"] => "Idle mode".to_string(),
                state_id!["NotShooting"] => "Not Shooting State".to_string(),
            },
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id![End] => state_id![],
                state_id!["Configuring"] => state_id![],
                state_id!["Idle"] => state_id![],
                state_id!["NotShooting"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id![End],
                    state_id!["Configuring"],
                    state_id!["Idle"],
                    state_id!["NotShooting"],
                },
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["Idle"]},
                    transition_id!{[Start]->["NotShooting"]},
                },
                state_id!["Configuring"] => btreeset! {
                    transition_id!{["Configuring"]->["Idle"]:"EvConfig"},
                },
                state_id!["Idle"] => btreeset! {
                    transition_id!{["Idle"]->["Configuring"]:"EvConfig"},
                },
                state_id!["NotShooting"] => btreeset! {
                    transition_id!{["NotShooting"]->[End]},
                },
            },
            state_transition_in: btreemap! {
                state_id![End] => btreeset! {
                    transition_id!{["NotShooting"]->[End]},
                },
                state_id!["Configuring"] => btreeset! {
                    transition_id!{["Idle"]->["Configuring"]:"EvConfig"},
                },
                state_id!["Idle"] => btreeset! {
                    transition_id!{[Start]->["Idle"]},
                    transition_id!{["Configuring"]->["Idle"]:"EvConfig"},
                },
                state_id!["NotShooting"] => btreeset! {
                    transition_id!{[Start]->["NotShooting"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["Idle"]} => state_id![Start],
                transition_id!{[Start]->["NotShooting"]} => state_id![Start],
                transition_id!{["Configuring"]->["Idle"]:"EvConfig"} => state_id!["Configuring"],
                transition_id!{["Idle"]->["Configuring"]:"EvConfig"} => state_id!["Idle"],
                transition_id!{["NotShooting"]->[End]} => state_id!["NotShooting"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["Idle"]} => state_id!["Idle"],
                transition_id!{[Start]->["NotShooting"]} => state_id!["NotShooting"],
                transition_id!{["Configuring"]->["Idle"]:"EvConfig"} => state_id!["Idle"],
                transition_id!{["Idle"]->["Configuring"]:"EvConfig"} => state_id!["Configuring"],
                transition_id!{["NotShooting"]->[End]} => state_id![End],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn skinparam_specifics() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            skinparam State {
              AttributeFontColor blue
              AttributeFontName serif
              AttributeFontSize  9
              AttributeFontStyle italic
              BackgroundColor palegreen
              BorderColor violet
              EndColor gold
              FontColor red
              FontName Sanserif
              FontSize 15
              FontStyle bold
              StartColor silver
            }
            
            state A : a a a\na
            state B : b b b\nb
            
            [*] -> A  : start
            A -> B : a2b
            B -> [*] : end
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_description: btreemap! {
                state_id!["A"] => vec![
                    "a a a\\na".to_string(),
                ],
                state_id!["B"] => vec![
                    "b b b\\nb".to_string(),
                ],
            },
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id![End] => state_id![],
                state_id!["A"] => state_id![],
                state_id!["B"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id![End],
                    state_id!["A"],
                    state_id!["B"],
                },
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["A"]:"start"},
                },
                state_id!["A"] => btreeset! {
                    transition_id!{["A"]->["B"]:"a2b"},
                },
                state_id!["B"] => btreeset! {
                    transition_id!{["B"]->[End]:"end"},
                },
            },
            state_transition_in: btreemap! {
                state_id![End] => btreeset! {
                    transition_id!{["B"]->[End]:"end"},
                },
                state_id!["A"] => btreeset! {
                    transition_id!{[Start]->["A"]:"start"},
                },
                state_id!["B"] => btreeset! {
                    transition_id!{["A"]->["B"]:"a2b"},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["A"]:"start"} => state_id![Start],
                transition_id!{["A"]->["B"]:"a2b"} => state_id!["A"],
                transition_id!{["B"]->[End]:"end"} => state_id!["B"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["A"]:"start"} => state_id!["A"],
                transition_id!{["A"]->["B"]:"a2b"} => state_id!["B"],
                transition_id!{["B"]->[End]:"end"} => state_id![End],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn style1() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            
            <style>
            stateDiagram {
              BackgroundColor Peru
              'LineColor Gray
              FontName Impact
              FontColor Red
              arrow {
                FontSize 13
                LineColor Blue
              }
            }
            </style>
            
            
            [*] --> NotShooting
            
            state "Not Shooting State" as NotShooting {
              state "Idle mode" as Idle <<Warning>>
              state "Configuring mode" as Configuring
              [*] --> Idle
              Idle --> Configuring : EvConfig
              Configuring --> Idle : EvConfig
            }
            
            NotShooting --> [*]
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_alias: btreemap! {
                state_id!["Configuring"] => "Configuring mode".to_string(),
                state_id!["Idle"] => "Idle mode".to_string(),
                state_id!["NotShooting"] => "Not Shooting State".to_string(),
            },
            state_parent: btreemap! {
                state_id![Start] => state_id![],
                state_id![End] => state_id![],
                state_id!["Configuring"] => state_id![],
                state_id!["Idle"] => state_id![],
                state_id!["NotShooting"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id![Start],
                    state_id![End],
                    state_id!["Configuring"],
                    state_id!["Idle"],
                    state_id!["NotShooting"],
                },
            },
            state_transition_out: btreemap! {
                state_id![Start] => btreeset! {
                    transition_id!{[Start]->["Idle"]},
                    transition_id!{[Start]->["NotShooting"]},
                },
                state_id!["Configuring"] => btreeset! {
                    transition_id!{["Configuring"]->["Idle"]:"EvConfig"},
                },
                state_id!["Idle"] => btreeset! {
                    transition_id!{["Idle"]->["Configuring"]:"EvConfig"},
                },
                state_id!["NotShooting"] => btreeset! {
                    transition_id!{["NotShooting"]->[End]},
                },
            },
            state_transition_in: btreemap! {
                state_id![End] => btreeset! {
                    transition_id!{["NotShooting"]->[End]},
                },
                state_id!["Configuring"] => btreeset! {
                    transition_id!{["Idle"]->["Configuring"]:"EvConfig"},
                },
                state_id!["Idle"] => btreeset! {
                    transition_id!{[Start]->["Idle"]},
                    transition_id!{["Configuring"]->["Idle"]:"EvConfig"},
                },
                state_id!["NotShooting"] => btreeset! {
                    transition_id!{[Start]->["NotShooting"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{[Start]->["Idle"]} => state_id![Start],
                transition_id!{[Start]->["NotShooting"]} => state_id![Start],
                transition_id!{["Configuring"]->["Idle"]:"EvConfig"} => state_id!["Configuring"],
                transition_id!{["Idle"]->["Configuring"]:"EvConfig"} => state_id!["Idle"],
                transition_id!{["NotShooting"]->[End]} => state_id!["NotShooting"],
            },
            transition_to: btreemap! {
                transition_id!{[Start]->["Idle"]} => state_id!["Idle"],
                transition_id!{[Start]->["NotShooting"]} => state_id!["NotShooting"],
                transition_id!{["Configuring"]->["Idle"]:"EvConfig"} => state_id!["Idle"],
                transition_id!{["Idle"]->["Configuring"]:"EvConfig"} => state_id!["Configuring"],
                transition_id!{["NotShooting"]->[End]} => state_id![End],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn style2() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            <style>
              diamond {
                BackgroundColor #palegreen
                LineColor #green
                LineThickness 2.5
            }
            </style>
            state state1
            state state2
            state choice1 <<choice>>
            state end3    <<end>>
            
            state1  --> choice1 : 1
            choice1 --> state2  : 2
            choice1 --> end3    : 3
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_stereotype: btreemap! {
                state_id!["choice1"] => StateStereoType::Choice,
                state_id!["end3"] => StateStereoType::Other(String::from("end")),
            },
            state_parent: btreemap! {
                state_id!["choice1"] => state_id![],
                state_id!["end3"] => state_id![],
                state_id!["state1"] => state_id![],
                state_id!["state2"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["choice1"],
                    state_id!["end3"],
                    state_id!["state1"],
                    state_id!["state2"],
                },
            },
            state_transition_out: btreemap! {
                state_id!["choice1"] => btreeset! {
                    transition_id!{["choice1"]->["end3"]:"3"},
                    transition_id!{["choice1"]->["state2"]:"2"},
                },
                state_id!["state1"] => btreeset! {
                    transition_id!{["state1"]->["choice1"]:"1"},
                },
            },
            state_transition_in: btreemap! {
                state_id!["choice1"] => btreeset! {
                    transition_id!{["state1"]->["choice1"]:"1"},
                },
                state_id!["end3"] => btreeset! {
                    transition_id!{["choice1"]->["end3"]:"3"},
                },
                state_id!["state2"] => btreeset! {
                    transition_id!{["choice1"]->["state2"]:"2"},
                },
            },
            transition_from: btreemap! {
                transition_id!{["choice1"]->["end3"]:"3"} => state_id!["choice1"],
                transition_id!{["choice1"]->["state2"]:"2"} => state_id!["choice1"],
                transition_id!{["state1"]->["choice1"]:"1"} => state_id!["state1"],
            },
            transition_to: btreemap! {
                transition_id!{["choice1"]->["end3"]:"3"} => state_id!["end3"],
                transition_id!{["choice1"]->["state2"]:"2"} => state_id!["state2"],
                transition_id!{["state1"]->["choice1"]:"1"} => state_id!["choice1"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn inline_style1() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state FooGradient #red-green ##00FFFF
            state FooDashed #red|green ##[dashed]blue {
            }
            state FooDotted ##[dotted]blue {
            }
            state FooBold ##[bold] {
            }
            state Foo1 ##[dotted]green {
            state inner1 ##[dotted]yellow
            }
            
            state out ##[dotted]gold
            
            state Foo2 ##[bold]green {
            state inner2 ##[dotted]yellow
            }
            inner1 -> inner2
            out -> inner2
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id!["Foo1"] => state_id![],
                state_id!["Foo1","inner1"] => state_id!["Foo1"],
                state_id!["Foo2"] => state_id![],
                state_id!["Foo2","inner2"] => state_id!["Foo2"],
                state_id!["FooBold"] => state_id![],
                state_id!["FooDashed"] => state_id![],
                state_id!["FooDotted"] => state_id![],
                state_id!["FooGradient"] => state_id![],
                state_id!["out"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["Foo1"],
                    state_id!["Foo2"],
                    state_id!["FooBold"],
                    state_id!["FooDashed"],
                    state_id!["FooDotted"],
                    state_id!["FooGradient"],
                    state_id!["out"],
                },
                state_id!["Foo1"] => btreeset! {
                    state_id!["Foo1","inner1"],
                },
                state_id!["Foo2"] => btreeset! {
                    state_id!["Foo2","inner2"],
                },
            },
            state_transition_out: btreemap! {
                state_id!["Foo1","inner1"] => btreeset! {
                    transition_id!{["Foo1","inner1"]->["Foo2","inner2"]},
                },
                state_id!["out"] => btreeset! {
                    transition_id!{["out"]->["Foo2","inner2"]},
                },
            },
            state_transition_in: btreemap! {
                state_id!["Foo2","inner2"] => btreeset! {
                    transition_id!{["Foo1","inner1"]->["Foo2","inner2"]},
                    transition_id!{["out"]->["Foo2","inner2"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{["Foo1","inner1"]->["Foo2","inner2"]} => state_id!["Foo1","inner1"],
                transition_id!{["out"]->["Foo2","inner2"]} => state_id!["out"],
            },
            transition_to: btreemap! {
                transition_id!{["Foo1","inner1"]->["Foo2","inner2"]} => state_id!["Foo2","inner2"],
                transition_id!{["out"]->["Foo2","inner2"]} => state_id!["Foo2","inner2"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn inline_style2() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state FooGradient #red-green;line:00FFFF
            state FooDashed #red|green;line.dashed;line:blue {
            }
            state FooDotted #line.dotted;line:blue {
            }
            state FooBold #line.bold {
            }
            state Foo1 #line.dotted;line:green {
            state inner1 #line.dotted;line:yellow
            }
            
            state out #line.dotted;line:gold
            
            state Foo2 #line.bold;line:green {
            state inner2 #line.dotted;line:yellow
            }
            inner1 -> inner2
            out -> inner2
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_parent: btreemap! {
                state_id!["Foo1"] => state_id![],
                state_id!["Foo1","inner1"] => state_id!["Foo1"],
                state_id!["Foo2"] => state_id![],
                state_id!["Foo2","inner2"] => state_id!["Foo2"],
                state_id!["FooBold"] => state_id![],
                state_id!["FooDashed"] => state_id![],
                state_id!["FooDotted"] => state_id![],
                state_id!["FooGradient"] => state_id![],
                state_id!["out"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["Foo1"],
                    state_id!["Foo2"],
                    state_id!["FooBold"],
                    state_id!["FooDashed"],
                    state_id!["FooDotted"],
                    state_id!["FooGradient"],
                    state_id!["out"],
                },
                state_id!["Foo1"] => btreeset! {
                    state_id!["Foo1","inner1"],
                },
                state_id!["Foo2"] => btreeset! {
                    state_id!["Foo2","inner2"],
                },
            },
            state_transition_out: btreemap! {
                state_id!["Foo1","inner1"] => btreeset! {
                    transition_id!{["Foo1","inner1"]->["Foo2","inner2"]},
                },
                state_id!["out"] => btreeset! {
                    transition_id!{["out"]->["Foo2","inner2"]},
                },
            },
            state_transition_in: btreemap! {
                state_id!["Foo2","inner2"] => btreeset! {
                    transition_id!{["Foo1","inner1"]->["Foo2","inner2"]},
                    transition_id!{["out"]->["Foo2","inner2"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{["Foo1","inner1"]->["Foo2","inner2"]} => state_id!["Foo1","inner1"],
                transition_id!{["out"]->["Foo2","inner2"]} => state_id!["out"],
            },
            transition_to: btreemap! {
                transition_id!{["Foo1","inner1"]->["Foo2","inner2"]} => state_id!["Foo2","inner2"],
                transition_id!{["out"]->["Foo2","inner2"]} => state_id!["Foo2","inner2"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn inline_style3() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state s1 : s1 description
            state s2 #pink;line:red;line.bold;text:red : s2 description
            state s3 #palegreen;line:green;line.dashed;text:green : s3 description
            state s4 #aliceblue;line:blue;line.dotted;text:blue   : s4 description
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_description: btreemap! {
                state_id!["s1"] => vec![
                    "s1 description".to_string(),
                ],
                state_id!["s2"] => vec![
                    "s2 description".to_string(),
                ],
                state_id!["s3"] => vec![
                    "s3 description".to_string(),
                ],
                state_id!["s4"] => vec![
                    "s4 description".to_string(),
                ],
            },
            state_parent: btreemap! {
                state_id!["s1"] => state_id![],
                state_id!["s2"] => state_id![],
                state_id!["s3"] => state_id![],
                state_id!["s4"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["s1"],
                    state_id!["s2"],
                    state_id!["s3"],
                    state_id!["s4"],
                },
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn alias1() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state alias1 
            state "alias2"
            state "long name" as alias3
            state alias4 as "long name"
            
            alias1 : ""state alias1""
            alias2 : ""state "alias2"""
            alias3 : ""state "long name" as alias3""
            alias4 : ""state alias4 as "long name"""
            
            alias1 -> alias2
            alias2 -> alias3
            alias3 -> alias4
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_alias: btreemap! {
                state_id!["alias3"] => "long name".to_string(),
                state_id!["alias4"] => "long name".to_string(),
            },
            state_description: btreemap! {
                state_id!["alias1"] => vec![
                    "state alias1".to_string(),
                ],
                state_id!["alias2"] => vec![
                    "state \"alias2\"".to_string(),
                ],
                state_id!["alias3"] => vec![
                    "state \"long name\" as alias3".to_string(),
                ],
                state_id!["alias4"] => vec![
                    "state alias4 as \"long name\"".to_string(),
                ],
            },
            state_parent: btreemap! {
                state_id!["alias1"] => state_id![],
                state_id!["alias2"] => state_id![],
                state_id!["alias3"] => state_id![],
                state_id!["alias4"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["alias1"],
                    state_id!["alias2"],
                    state_id!["alias3"],
                    state_id!["alias4"],
                },
            },
            state_transition_out: btreemap! {
                state_id!["alias1"] => btreeset! {
                    transition_id!{["alias1"]->["alias2"]},
                },
                state_id!["alias2"] => btreeset! {
                    transition_id!{["alias2"]->["alias3"]},
                },
                state_id!["alias3"] => btreeset! {
                    transition_id!{["alias3"]->["alias4"]},
                },
            },
            state_transition_in: btreemap! {
                state_id!["alias2"] => btreeset! {
                    transition_id!{["alias1"]->["alias2"]},
                },
                state_id!["alias3"] => btreeset! {
                    transition_id!{["alias2"]->["alias3"]},
                },
                state_id!["alias4"] => btreeset! {
                    transition_id!{["alias3"]->["alias4"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{["alias1"]->["alias2"]} => state_id!["alias1"],
                transition_id!{["alias2"]->["alias3"]} => state_id!["alias2"],
                transition_id!{["alias3"]->["alias4"]} => state_id!["alias3"],
            },
            transition_to: btreemap! {
                transition_id!{["alias1"]->["alias2"]} => state_id!["alias2"],
                transition_id!{["alias2"]->["alias3"]} => state_id!["alias3"],
                transition_id!{["alias3"]->["alias4"]} => state_id!["alias4"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn alias2() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state alias1 : ""state alias1""
            state "alias2" : ""state "alias2"""
            state "long name" as alias3 : ""state "long name" as alias3""
            state alias4 as "long name" : ""state alias4 as "long name"""
            
            alias1 -> alias2
            alias2 -> alias3
            alias3 -> alias4
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_alias: btreemap! {
                state_id!["alias3"] => "long name".to_string(),
                state_id!["alias4"] => "long name".to_string(),
            },
            state_description: btreemap! {
                state_id!["alias1"] => vec![
                    "state alias1".to_string(),
                ],
                state_id!["alias2"] => vec![
                    "state \"alias2\"".to_string(),
                ],
                state_id!["alias3"] => vec![
                    "state \"long name\" as alias3".to_string(),
                ],
                state_id!["alias4"] => vec![
                    "state alias4 as \"long name\"".to_string(),
                ],
            },
            state_parent: btreemap! {
                state_id!["alias1"] => state_id![],
                state_id!["alias2"] => state_id![],
                state_id!["alias3"] => state_id![],
                state_id!["alias4"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["alias1"],
                    state_id!["alias2"],
                    state_id!["alias3"],
                    state_id!["alias4"],
                },
            },
            state_transition_out: btreemap! {
                state_id!["alias1"] => btreeset! {
                    transition_id!{["alias1"]->["alias2"]},
                },
                state_id!["alias2"] => btreeset! {
                    transition_id!{["alias2"]->["alias3"]},
                },
                state_id!["alias3"] => btreeset! {
                    transition_id!{["alias3"]->["alias4"]},
                },
            },
            state_transition_in: btreemap! {
                state_id!["alias2"] => btreeset! {
                    transition_id!{["alias1"]->["alias2"]},
                },
                state_id!["alias3"] => btreeset! {
                    transition_id!{["alias2"]->["alias3"]},
                },
                state_id!["alias4"] => btreeset! {
                    transition_id!{["alias3"]->["alias4"]},
                },
            },
            transition_from: btreemap! {
                transition_id!{["alias1"]->["alias2"]} => state_id!["alias1"],
                transition_id!{["alias2"]->["alias3"]} => state_id!["alias2"],
                transition_id!{["alias3"]->["alias4"]} => state_id!["alias3"],
            },
            transition_to: btreemap! {
                transition_id!{["alias1"]->["alias2"]} => state_id!["alias2"],
                transition_id!{["alias2"]->["alias3"]} => state_id!["alias3"],
                transition_id!{["alias3"]->["alias4"]} => state_id!["alias4"],
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}

#[test]
fn json1() -> anyhow::Result<()> {
    let data = r#"
            @startuml
            state "A" as stateA
            state "C" as stateC {
             state B
            }
            
            json jsonJ {
               "fruit":"Apple",
               "size":"Large",
               "color": ["Red", "Green"]
            }
            @enduml
        "#;

    let (input, diagram) = human_readable_error(mermaid)(data)?;
    assert!(input.is_empty());
    assert_eq!(
        Diagram {
            state_alias: btreemap! {
                state_id!["stateA"] => "A".to_string(),
                state_id!["stateC"] => "C".to_string(),
            },
            state_parent: btreemap! {
                state_id!["B"] => state_id![],
                state_id!["stateA"] => state_id![],
                state_id!["stateC"] => state_id![],
            },
            state_children: btreemap! {
                state_id![] => btreeset! {
                    state_id!["B"],
                    state_id!["stateA"],
                    state_id!["stateC"],
                },
            },
            ..Default::default()
        },
        diagram,
    );
    Ok(())
}
