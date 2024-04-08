use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::{Debug, Display, Formatter};

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone)]
pub enum Frame {
    #[default]
    Start,
    End,
    History,
    DeepHistory,
    State {
        name: String,
    },
}

impl Debug for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "frame![")?;
        match self {
            Frame::Start => write!(f, "{}", "Start")?,
            Frame::End => write!(f, "{}", "End")?,
            Frame::History => write!(f, "{}", "History")?,
            Frame::DeepHistory => write!(f, "{}", "DeepHistory")?,
            Frame::State { name } => write!(f, "\"{}\"", name)?,
        }
        write!(f, "]")?;
        Ok(())
    }
}

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone)]
pub struct Frames {
    pub frames: VecDeque<Frame>,
}

impl Debug for Frames {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Frames {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "frames![")?;
        let mut iter = self.frames.iter();
        if let Some(frame) = iter.next() {
            match frame {
                Frame::Start => write!(f, "{}", "Start")?,
                Frame::End => write!(f, "{}", "End")?,
                Frame::History => write!(f, "{}", "History")?,
                Frame::DeepHistory => write!(f, "{}", "DeepHistory")?,
                Frame::State { name } => write!(f, "\"{}\"", name)?,
            }
        }
        for frame in iter {
            match frame {
                Frame::Start => write!(f, "{}", ",Start")?,
                Frame::End => write!(f, "{}", ",End")?,
                Frame::History => write!(f, "{}", ",History")?,
                Frame::DeepHistory => write!(f, "{}", ",DeepHistory")?,
                Frame::State { name } => write!(f, ",\"{}\"", name)?,
            };
        }
        write!(f, "]")?;
        Ok(())
    }
}

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone)]
pub struct StateId(pub VecDeque<Frame>);

impl Debug for StateId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for StateId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "state_id![")?;
        let mut iter = self.0.iter();
        match iter.next() {
            Some(Frame::Start) => write!(f, "{}", "Start")?,
            Some(Frame::End) => write!(f, "{}", "End")?,
            Some(Frame::History) => write!(f, "{}", "History")?,
            Some(Frame::DeepHistory) => write!(f, "{}", "DeepHistory")?,
            Some(Frame::State { name }) => write!(f, "\"{}\"", name)?,
            None => {}
        }
        for frame in iter {
            match frame {
                Frame::Start => write!(f, "{}", ",Start")?,
                Frame::End => write!(f, "{}", ",End")?,
                Frame::History => write!(f, "{}", ",History")?,
                Frame::DeepHistory => write!(f, "{}", ",DeepHistory")?,
                Frame::State { name } => write!(f, ",\"{}\"", name)?,
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone)]
pub struct TransitionId(pub StateId, pub StateId, pub Option<String>);

impl Debug for TransitionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for TransitionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "transition_id!{{[")?;
        let mut iter = self.0 .0.iter();
        match iter.next() {
            Some(Frame::Start) => write!(f, "{}", "Start")?,
            Some(Frame::End) => write!(f, "{}", "End")?,
            Some(Frame::History) => write!(f, "{}", "History")?,
            Some(Frame::DeepHistory) => write!(f, "{}", "DeepHistory")?,
            Some(Frame::State { name }) => write!(f, "\"{}\"", name)?,
            None => {}
        }
        for frame in iter {
            match frame {
                Frame::Start => write!(f, "{}", ",Start")?,
                Frame::End => write!(f, "{}", ",End")?,
                Frame::History => write!(f, "{}", ",History")?,
                Frame::DeepHistory => write!(f, "{}", ",DeepHistory")?,
                Frame::State { name } => write!(f, ",\"{}\"", name)?,
            }
        }
        write!(f, "]->[")?;
        let mut iter = self.1 .0.iter();
        match iter.next() {
            Some(Frame::Start) => write!(f, "{}", "Start")?,
            Some(Frame::End) => write!(f, "{}", "End")?,
            Some(Frame::History) => write!(f, "{}", "History")?,
            Some(Frame::DeepHistory) => write!(f, "{}", "DeepHistory")?,
            Some(Frame::State { name }) => write!(f, "\"{}\"", name)?,
            None => {}
        }
        for frame in iter {
            match frame {
                Frame::Start => write!(f, "{}", ",Start")?,
                Frame::End => write!(f, "{}", ",End")?,
                Frame::History => write!(f, "{}", ",History")?,
                Frame::DeepHistory => write!(f, "{}", ",DeepHistory")?,
                Frame::State { name } => write!(f, ",\"{}\"", name)?,
            }
        }
        if let Some(desc) = &self.2 {
            write!(f, "]:\"{}\"}}", desc)?;
        } else {
            write!(f, "]}}")?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! state_id {
    ($($x:tt)*) => {
        StateId(frames![$($x)*].frames)
    };
}

#[macro_export]
macro_rules! transition_id {
    ([$($from:tt)*] -> [$($to:tt)*]: $desc:expr) => {
        TransitionId(state_id![$($from)*], state_id![$($to)*], Some($desc.to_string()))
    };
    ([$($from:tt)*] -> [$($to:tt)*]) => {
        TransitionId(state_id![$($from)*], state_id![$($to)*], None)
    };
}

#[macro_export]
macro_rules! frame {
    ($ident:ident) => {{
        Frame::$ident
    }};
    ($name:expr) => {{
        Frame::State {
            name: String::from($name),
        }
    }};
}

#[macro_export]
macro_rules! frames {
    // Base case: empty call
    () => {
        Frames {
            frames: VecDeque::new(),
        }
    };

    ($ident:ident $(, $rest:tt)*) => {
        {
            let mut frames = frames![$($rest),*].frames;
            frames.push_front(frame![$ident]);
            Frames { frames }
        }
    };

    ($name:expr $(, $rest:tt)*) => {
        {
            let mut frames = frames![$($rest),*].frames;
            frames.push_front(frame![$name]);
            Frames { frames }
        }
    };
}

#[derive(Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone)]
pub enum StateStereoType {
    #[default]
    Start,
    End,
    Choice,
    Fork,
    Join,
    SdlReceive,
    EntryPoint,
    ExitPoint,
    InputPin,
    OutputPin,
    ExpansionInput,
    ExpansionOutput,
    Other(String),
}

#[derive(Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone)]
pub struct Diagram {
    pub state_alias: BTreeMap<StateId, String>,
    pub state_note: BTreeMap<StateId, Vec<String>>,
    pub state_description: BTreeMap<StateId, Vec<String>>,
    pub state_stereotype: BTreeMap<StateId, StateStereoType>,
    pub state_parent: BTreeMap<StateId, StateId>,
    pub state_children: BTreeMap<StateId, BTreeSet<StateId>>,
    pub state_children_are_concurrent: BTreeSet<StateId>,
    pub state_transition_out: BTreeMap<StateId, BTreeSet<TransitionId>>,
    pub state_transition_in: BTreeMap<StateId, BTreeSet<TransitionId>>,
    pub transition_from: BTreeMap<TransitionId, StateId>,
    pub transition_to: BTreeMap<TransitionId, StateId>,
    pub transition_note: BTreeMap<TransitionId, Vec<String>>,
    pub note: Vec<String>,
}

impl Diagram {
    pub fn is_empty(&self) -> bool {
        self.eq(&Self::default())
    }
}
