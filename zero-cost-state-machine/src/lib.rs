pub trait Take<S, C> {
    type Destination;
    /// produces the state associated with the course out of the given state
    fn take(&self, course: C, state: S) -> Self::Destination;
}

pub trait Stop<S, C> {
    type Destination;
    type History;
    /// stops the current state, leaving behind a history that could be resumed
    fn stop(&self, course: C, state: S) -> (Self::Destination, Self::History);
}

pub trait Resume<S, H, C> {
    type Destination;
    /// resumes from the given history by leaving the state along the given course
    fn resume(&self, course: C, state: S, history: H) -> Self::Destination;
}

/// Represents the start of a machine
pub struct Start;

/// Represents the end of a machine
pub struct End;

pub trait History {
    /// Represents the history of a machine
    type History;
}

pub trait DeepHistory {
    /// Represents the deep history of a machine
    type DeepHistory;
}

pub trait Switch<P> {
    type Target;
    fn transition(self, path: P) -> Self::Target;
}

//
// trait Foo
// where
//     Self: Take<(), (), Destination = ()>,
// {
// }
//
// // trait Descend<P> {
// //     type Child<'a>;
// //     fn transition<'a>(&'a self, path: P) -> Self::Child<'a>;
// // }
//
// pub mod edge {
//     pub struct Down;
// }
// pub mod node {
//     pub struct Initial;
// }
//
// // TODO maybe instead of Switch, use Div and DivAssign
// pub trait Switch<P> {
//     type Target;
//     fn transition(self, path: P) -> Self::Target;
// }
//
// pub struct State<S> {
//     pub state: S,
// }
//
// impl Switch<edge::Down> for State<node::Initial> {
//     type Target = foo::State<node::Initial, edge::Down, foo::node::A>;
//
//     fn transition(self, path: edge::Down) -> Self::Target {
//         foo::State {
//             state0: self.state,
//             path0: path,
//             state: foo::node::A,
//         }
//     }
// }
// /// TODO handle a case for switching out of a nested state into another nested state
// ///  maybe instead of descend and switch, just have switch
// mod foo {
//     use crate::Switch;
//     pub mod edge {
//         pub struct Sideways;
//         pub struct Backwards;
//         pub struct Down;
//     }
//     pub mod node {
//         pub struct A;
//         pub struct B;
//     }
//     pub struct State<S0, P0, S> {
//         pub state0: S0,
//         pub path0: P0,
//         pub state: S,
//     }
//     impl<S0, P0> Switch<edge::Sideways> for State<S0, P0, node::A> {
//         type Target = State<S0, P0, node::B>;
//         fn transition(self, _: edge::Sideways) -> Self::Target {
//             State {
//                 state0: self.state0,
//                 path0: self.path0,
//                 state: node::B,
//             }
//         }
//     }
//     impl<S0, P0> Switch<edge::Backwards> for State<S0, P0, node::B> {
//         type Target = State<S0, P0, node::A>;
//         fn transition(self, _: &edge::Backwards) -> Self::Target {
//             State {
//                 state0: self.state0,
//                 path0: self.path0,
//                 state: node::A,
//             }
//         }
//     }
//     impl<S0, P0> Switch<edge::Down> for State<S0, P0, node::B> {
//         type Target = b::State<S0, P0, node::B, edge::Down, b::node::C>;
//         fn transition(self, path: edge::Down) -> Self::Target {
//             b::State {
//                 state0: self.state0,
//                 path0: self.path0,
//                 state1: node::B,
//                 path1: path,
//                 state: b::node::C,
//             }
//         }
//     }
//     // impl<S0, P0> Descend<transition::Down> for State<S0, P0, state::B> {
//     //     type Child<'a> = b::State<S0, P0, state::B, transition::Down, b::state::C>;
//     //
//     //     fn transition<'a>(self, path: transition::Down) -> Self::Child<'a> {
//     //         b::State {
//     //             state0: &self.state0,
//     //             path0: self.path0,
//     //             state1: state::B,
//     //             path1: path,
//     //             state: b::state::C,
//     //         }
//     //     }
//     // }
//     pub mod b {
//         use crate::Switch;
//         pub mod edge {
//             pub struct Sideways;
//             pub struct CrossWays;
//         }
//         pub mod node {
//             pub struct C;
//             pub struct D;
//         }
//         pub struct State<S0, P0, S1, P1, S> {
//             pub state0: S0,
//             pub path0: P0,
//             pub state1: S1,
//             pub path1: P1,
//             pub state: S,
//         }
//         impl<S0, P0, S1, P1> Switch<edge::Sideways> for State<S0, P0, S1, P1, node::C> {
//             type Target = State<S0, P0, S1, P1, node::D>;
//             fn transition(self, _: edge::Sideways) -> Self::Target {
//                 State {
//                     state0: self.state0,
//                     path0: self.path0,
//                     state1: self.state1,
//                     path1: self.path1,
//                     state: node::D,
//                 }
//             }
//         }
//         impl<S0, P0, S1, P1> Switch<edge::CrossWays> for State<S0, P0, S1, P1, node::D> {
//             type Target = super::State<S0, P0, super::node::A>;
//             fn transition(self, _: edge::CrossWays) -> Self::Target {
//                 super::State {
//                     state0: self.state0,
//                     path0: self.path0,
//                     state: super::node::A,
//                 }
//             }
//         }
//     }
// }
//
// impl State<node::Initial> {
//     fn init(self) {
//         let var1 = 32;
//         let var2 = "foo";
//         let s = self.transition(edge::Down);
//         s.operate(var1, var2);
//     }
// }
//
// impl foo::State<node::Initial, edge::Down, foo::node::A> {
//     fn operate(self, v1: usize, v2: &'static str) {
//         unimplemented!()
//     }
// }
//
// impl foo::b::State<node::Initial, edge::Down, foo::node::B, foo::edge::Down, foo::b::node::C> {
//     fn operate(self) {
//         unimplemented!()
//     }
// }
//
// #[cfg(test)]
// mod test {
//     use crate::foo::edge;
//     use crate::{edge, foo, node, State, Switch};
//     #[test]
//     fn foo() {
//         let m = State {
//             state: &node::Initial,
//         };
//         m.init();
//         // m.stuff();
//         // let m = m.transition(&edge::Down);
//         // m.more_stuff();
//         // let m = m.transition(&foo::edge::Sideways);
//         // let m = m.transition(&foo::edge::Down);
//         // m.even_more_stuff();
//         // let m = m.transition(&foo::b::edge::Sideways);
//         // let m = m.transition(&foo::b::edge::CrossWays);
//     }
// }
