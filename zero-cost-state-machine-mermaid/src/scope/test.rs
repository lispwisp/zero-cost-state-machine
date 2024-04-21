use crate::scope::Scope;
use crate::scope::Tree;
use crate::{tree, Frame};
use internal::{frame, frames, Frames};
use maplit::btreemap;
use pretty_assertions::assert_eq;
use std::collections::VecDeque;

#[test]
fn basic() {
    let mut scope: Scope = Default::default();
    scope.insert(frames!["foo", "bar", Start].frames);
    assert_eq!(
        Scope {
            frame_tree: tree![
                "foo" => tree![
                    "bar" => tree![
                        Start => tree![]
                    ]
                ]
            ],
            context_resume: btreemap![
                frame!("foo") => frames![].frames,
                frame!("bar") => frames!["foo"].frames,
            ]
        },
        scope
    );
}

#[test]
fn no_overlap() {
    let mut scope: Scope = Default::default();
    scope.insert(frames!["foo", "bar", Start].frames);
    scope.insert(frames!["baz", "bar", End].frames);
    assert_eq!(
        Scope {
            frame_tree: tree![
                "foo" => tree![
                    "bar" => tree![
                        Start => tree![]
                    ]
                ],
                "baz" => tree![
                  "bar" => tree![
                      End => tree![]
                  ]
                ]
            ],
            context_resume: btreemap! {
                frame!("foo") => frames![].frames,
                frame!("bar") => frames!["foo"].frames,
                frame!("baz") => frames![].frames,
            }
        },
        scope
    );
}

#[test]
fn front_mid_overlap() {
    let mut scope: Scope = Default::default();
    scope.insert(frames!["foo", "bar", Start].frames);
    scope.insert(frames!["foo", "bar", End].frames);
    assert_eq!(
        Scope {
            frame_tree: tree![
                "foo" => tree![
                    "bar" => tree![
                        Start => tree![],
                        End => tree![],
                    ]
                ],
            ],
            context_resume: btreemap! {
                frame!("foo") => frames![].frames,
                frame!("bar") => frames!["foo"].frames,
            }
        },
        scope
    );
}

#[test]
fn mid_overlap() {
    let mut scope: Scope = Default::default();
    scope.insert(frames!["foo", "baz", Start].frames);
    scope.insert(frames!["bar", "baz", End].frames);
    assert_eq!(
        Scope {
            frame_tree: tree![
                "foo" => tree![
                    "baz" => tree![
                        Start => tree![],
                    ]
                ],
                "bar" => tree![
                    "baz" => tree![
                        End => tree![],
                    ]
                ],
            ],
            context_resume: btreemap! {
                frame!("foo") => frames![].frames,
                frame!("baz") => frames!["foo"].frames,
                frame!("bar") => frames![].frames,
            }
        },
        scope
    );
}

#[test]
fn total_overlap() {
    let mut scope: Scope = Default::default();
    scope.insert(frames!["foo", "baz", Start].frames);
    scope.insert(frames!["foo", "baz", Start].frames);
    assert_eq!(
        Scope {
            frame_tree: tree![
                "foo" => tree![
                    "baz" => tree![
                        Start => tree![],
                    ]
                ],
            ],
            context_resume: btreemap! {
                frame!("foo") => frames![].frames,
                frame!("baz") => frames!["foo"].frames,
            }
        },
        scope
    );
}

#[test]
fn deepening() {
    let mut scope: Scope = Default::default();
    scope.insert(frames!["foo", "baz", "qux"].frames);
    scope.insert(frames!["foo", "baz", "qux", End].frames);
    assert_eq!(
        Scope {
            frame_tree: tree![
                "foo" => tree![
                    "baz" => tree![
                        "qux" => tree![
                            End => tree![]
                        ]
                    ]
                ],
            ],
            context_resume: btreemap! {
                frame!("foo") => frames![].frames,
                frame!("baz") => frames!["foo"].frames,
                frame!("qux") => frames!["foo", "baz"].frames,
            }
        },
        scope
    );
}

#[test]
fn cycling() {
    let mut scope: Scope = Default::default();
    scope.insert(frames!["foo", "baz", "qux"].frames);
    scope.insert(frames!["foo", "baz", "foo"].frames);
    assert_eq!(
        Scope {
            frame_tree: tree![
                "foo" => tree![
                    "baz" => tree![
                        "qux" => tree![],
                        "foo" => tree![]
                    ]
                ],
            ],
            context_resume: btreemap! {
                frame!("foo") => frames![].frames,
                frame!("baz") => frames!["foo"].frames,
                frame!("qux") => frames!["foo", "baz"].frames,
            }
        },
        scope
    );
}

#[test]
fn restore() {
    let mut scope: Scope = Default::default();
    scope.insert(frames!["foo", "baz", "qux"].frames);
    let frame_stack = frames![].frames;
    let mut context = frames!["qux", "qix"].frames;
    scope.restore_context(&frame_stack, &mut context);
    assert_eq!(frames!["foo", "baz", "qux", "qix"].frames, context);
}

#[test]
fn resume() {
    let mut scope: Scope = Default::default();
    scope.insert(frames!["foo", "baz", "qux"].frames);
    let frame_stack = frames![].frames;
    let mut frames = frames!["baz", "bar"].frames;
    scope.resume_or_insert(&frame_stack, &mut frames);
    assert_eq!(
        Scope {
            frame_tree: tree![
                "foo" => tree![
                    "baz" => tree![
                        "qux" => tree![],
                        "bar" => tree![]
                    ]
                ],
            ],
            context_resume: btreemap! {
                frame!("foo") => frames![].frames,
                frame!("baz") => frames!["foo"].frames,
                frame!("qux") => frames!["foo", "baz"].frames,
                frame!("bar") => frames!["foo", "baz"].frames,
            }
        },
        scope
    );
}

#[test]
fn resume_with_cycle() {
    let mut scope: Scope = Default::default();
    scope.insert(frames!["foo", "baz", "qux"].frames);
    let frame_stack = frames![].frames;
    let mut frames = frames!["qux", "foo"].frames;
    scope.resume_or_insert(&frame_stack, &mut frames);
    let mut frames = frames!["foo", "bar"].frames;
    scope.resume_or_insert(&frame_stack, &mut frames);
    assert_eq!(
        Scope {
            frame_tree: tree![
                "foo" => tree![
                    "baz" => tree![
                        "qux" => tree![
                            "foo" => tree![]
                        ],
                    ],
                    "bar" => tree![]
                ],
            ],
            context_resume: btreemap! {
                frame!("foo") => frames![].frames,
                frame!("baz") => frames!["foo"].frames,
                frame!("qux") => frames!["foo", "baz"].frames,
                frame!("bar") => frames!["foo"].frames,
            }
        },
        scope
    );
}
