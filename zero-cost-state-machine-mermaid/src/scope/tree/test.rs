use crate::scope::Tree;
use internal::frame;
use internal::frames;
use internal::Frame;
use internal::Frames;
use pretty_assertions::assert_eq;
use std::collections::VecDeque;

#[test]
fn basic() {
    let mut tree: Tree = Default::default();
    tree.insert(frames!["foo", "bar", Start].frames);
    assert_eq!(
        tree![
            "foo" => tree![
                "bar" => tree![
                    Start => tree![]
                ]
            ]
        ],
        tree
    );
}

#[test]
fn no_overlap() {
    let mut tree: Tree = Default::default();
    tree.insert(frames!["foo", "bar", Start].frames);
    tree.insert(frames!["baz", "bar", End].frames);
    assert_eq!(
        tree![
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
        tree
    );
}

#[test]
fn front_mid_overlap() {
    let mut tree: Tree = Default::default();
    tree.insert(frames!["foo", "bar", Start].frames);
    tree.insert(frames!["foo", "bar", End].frames);
    assert_eq!(
        tree![
            "foo" => tree![
                "bar" => tree![
                    Start => tree![],
                    End => tree![],
                ]
            ],
        ],
        tree
    );
}

#[test]
fn mid_overlap() {
    let mut tree: Tree = Default::default();
    tree.insert(frames!["foo", "baz", Start].frames);
    tree.insert(frames!["bar", "baz", End].frames);
    assert_eq!(
        tree![
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
        tree
    );
}

#[test]
fn total_overlap() {
    let mut tree: Tree = Default::default();
    tree.insert(frames!["foo", "baz", Start].frames);
    tree.insert(frames!["foo", "baz", Start].frames);
    assert_eq!(
        tree![
            "foo" => tree![
                "baz" => tree![
                    Start => tree![],
                ]
            ],
        ],
        tree
    );
}

#[test]
fn deepening() {
    let mut tree: Tree = Default::default();
    tree.insert(frames!["foo", "baz", "qux"].frames);
    tree.insert(frames!["foo", "baz", "qux", End].frames);
    assert_eq!(
        tree![
            "foo" => tree![
                "baz" => tree![
                    "qux" => tree![
                        End => tree![],
                    ]
                ]
            ],
        ],
        tree
    );
}

#[test]
fn flatten() {
    let mut tree: Tree = Default::default();
    tree.insert(frames!["foo", "bar", Start].frames);
    tree.insert(frames!["foo", "baz"].frames);
    tree.insert(frames!["bar", "bar", "foo"].frames);
    tree.insert(frames!["foo", "bar", End].frames);
    assert_eq!(
        tree![
            "bar" => tree![
                "bar" => tree![
                    "foo" => tree![],
                ],
            ],
            "foo" => tree![
                "bar" => tree![
                    Start => tree![],
                    End => tree![],
                ],
                "baz" => tree![],
            ],
        ],
        tree
    );
    let flattened = tree.flatten();
    assert_eq!(
        vec![
            frames!["bar", "bar", "foo"].frames,
            frames!["foo", "bar", Start].frames,
            frames!["foo", "bar", End].frames,
            frames!["foo", "baz"].frames,
        ],
        flattened
    );
}
