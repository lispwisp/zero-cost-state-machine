use crate::scope::tree::Tree;
use crate::Frame;
use std::collections::{BTreeMap, VecDeque};
pub mod tree;

#[derive(Default, Debug, PartialEq)]
pub struct Scope {
    pub frame_tree: Tree,
    pub context_resume: BTreeMap<Frame, VecDeque<Frame>>,
}

impl Scope {
    fn restore_context(&self, frame_stack: &VecDeque<Frame>, frames: &mut VecDeque<Frame>) {
        let top_binding_point = frames.pop_front();
        if let Some(binding_point) = top_binding_point {
            let skip = match &binding_point {
                Frame::Start => frame_stack.len() > 0,
                Frame::End => frame_stack.len() > 0,
                Frame::History => frame_stack.len() > 0,
                Frame::DeepHistory => frame_stack.len() > 0,
                _ => false,
            };
            match (skip, self.context_resume.get(&binding_point)) {
                (false, Some(existing_context)) => {
                    frames.push_front(binding_point);
                    for frame in existing_context.iter().rev() {
                        frames.push_front(frame.clone());
                    }
                }
                _ => {
                    frames.push_front(binding_point);
                    for frame in frame_stack.iter().rev() {
                        frames.push_front(frame.clone());
                    }
                }
            }
        }
    }
    fn save_context(&mut self, frames: &[Frame]) {
        if frames.len() == 0 {
            return;
        }
        let (frames, bottom_binding_point) = frames.split_at(frames.len() - 1);
        let bottom_binding_point = bottom_binding_point[0].clone();
        self.context_resume
            .entry(bottom_binding_point)
            .or_insert(frames.iter().cloned().collect());
    }
    pub fn insert(&mut self, mut frames: VecDeque<Frame>) {
        self.frame_tree.insert(frames.clone());
        let frames = frames.make_contiguous();
        let l = match &frames.last() {
            Some(&Frame::Start) => frames.len() - 1,
            Some(&Frame::End) => frames.len() - 1,
            Some(&Frame::History) => frames.len() - 1,
            Some(&Frame::DeepHistory) => frames.len() - 1,
            _ => frames.len(),
        };
        for i in 0..l {
            self.save_context(&frames[..=i]);
        }
    }
    pub fn resume_or_insert(
        &mut self,
        frame_stack: &VecDeque<Frame>,
        frames: &mut VecDeque<Frame>,
    ) {
        self.restore_context(frame_stack, frames);
        self.insert(frames.clone());
    }
    pub fn flatten(self) -> Vec<VecDeque<Frame>> {
        self.frame_tree.flatten()
    }
}

#[cfg(test)]
mod test;
