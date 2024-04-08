use zero_cost_state_machine::Switch;
use zero_cost_state_machine_macro::statemachine_from_puml;

// statemachine_from_puml! {
//     r#"@startuml
//     scale 600 width
//
//     [*] -> State1
//     State1 --> State2 : Succeeded
//     State1 --> [*] : Aborted
//     State2 --> State3 : Succeeded
//     State2 --> [*] : Aborted
//     state State3 {
//       state "Accumulate Enough Data\nLong State Name" as long1
//       long1 : Just a test
//       [*] --> long1
//       long1 --> long1 : New Data
//       long1 --> ProcessData : Enough Data
//       long1 --> [*]
//     }
//     State3 --> State3 : Failed
//     State3 --> [*] : Succeeded / Save Result
//     State3 --> [*] : Aborted
//
//     @enduml"#
// }

#[test]
fn foo() -> anyhow::Result<()> {
    Ok(())
}
