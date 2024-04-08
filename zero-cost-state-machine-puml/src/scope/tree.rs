use crate::Frame;
use std::collections::{BTreeMap, VecDeque};
use std::fmt::{Debug, Display, Formatter};

#[macro_export]
macro_rules! tree {
    () => {
        Tree::default()
    };

    // Recursive case: Match and process multiple key-value pairs
    ($($key:ident => $val:expr),+ $(,)?) => {
        {
            let mut temp_map = std::collections::BTreeMap::new();
            $(
                temp_map.insert(Frame::$key, Box::new($val));
            )+
            Tree {
                children: temp_map
            }
        }
    };

    ($($key:expr => $val:expr),+ $(,)?) => {
        {
            let mut temp_map = std::collections::BTreeMap::new();
            $(
                temp_map.insert(Frame::State { name: String::from($key) }, Box::new($val));
            )+
            Tree {
                children: temp_map
            }
        }
    };
}

#[derive(Default, PartialEq)]
pub struct Tree {
    pub children: BTreeMap<Frame, Box<Tree>>,
}

impl Debug for Tree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Tree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn print_tree(tree: &Tree, f: &mut Formatter<'_>, depth: usize) -> std::fmt::Result {
            if tree.children.is_empty() {
                return write!(f, "tree![],\n");
            }
            write!(f, "tree![\n")?;
            let indent = "\t".repeat(depth + 1);
            for (k, v) in &tree.children {
                write!(f, "{}", indent)?;
                match k {
                    Frame::Start => write!(f, "Start => ")?,
                    Frame::End => write!(f, "End => ")?,
                    Frame::History => write!(f, "History => ")?,
                    Frame::DeepHistory => write!(f, "DeepHistory => ")?,
                    Frame::State { name } => write!(f, "\"{}\" => ", name)?,
                }
                print_tree(v, f, depth + 1)?;
            }
            write!(
                f,
                "{}]{}",
                "\t".repeat(depth),
                if depth == 0 { "" } else { ",\n" }
            )
        }

        print_tree(self, f, 0)
    }
}

impl Tree {
    pub fn insert(&mut self, mut frames: VecDeque<Frame>) {
        if let Some(f) = frames.pop_front() {
            self.children
                .entry(f)
                .or_insert(Default::default())
                .insert(frames);
        }
    }
    pub fn flatten(mut self) -> Vec<VecDeque<Frame>> {
        fn depth_first(
            tree: &mut Tree,
            log: &mut Vec<VecDeque<Frame>>,
            stack: &mut VecDeque<Frame>,
        ) {
            if tree.children.is_empty() {
                log.push(stack.clone());
            }
            for (k, mut v) in &mut tree.children {
                stack.push_back(k.clone());
                depth_first(&mut v, log, stack);
            }
            stack.pop_back();
        }
        let mut v = vec![];
        let mut s = VecDeque::new();
        depth_first(&mut self, &mut v, &mut s);
        v
    }
}

#[cfg(test)]
mod test;
