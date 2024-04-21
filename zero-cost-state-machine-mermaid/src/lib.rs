use crate::scope::Scope;
use anyhow::anyhow;
use nom::branch::alt;
use nom::bytes::complete::{
    escaped_transform, is_not, tag, tag_no_case, take_till, take_until, take_while1,
};
use nom::character::complete::{anychar, line_ending, multispace0, space0, space1};
use nom::combinator::{fail, map, opt, peek, recognize, value};
use nom::error::{convert_error, VerboseError};
use nom::multi::{many0, many_till, separated_list1};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::ops::Deref;

pub use internal::*;

mod scope;

#[cfg(test)]
mod tests;

fn quoted_string(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    delimited(
        tag("\""),
        escaped_transform(
            take_while1(|c: char| c != '\\' && c != '"'),
            '\\',
            alt((
                value("\\", tag("\\")),
                value("\"", tag("\"")),
                value("\t", tag("\t")),
                value("\n", tag("n")),
            )),
        ),
        tag("\""),
    )(input)
}

fn token1(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}

fn token1_maybe_quote(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    alt((delimited(tag("\""), token1, tag("\"")), token1))(input)
}

fn scoped_token1(input: &str) -> IResult<&str, Vec<&str>, VerboseError<&str>> {
    separated_list1(tag("."), token1)(input)
}

fn scoped_token1_maybe_quote(input: &str) -> IResult<&str, Vec<&str>, VerboseError<&str>> {
    alt((
        delimited(tag("\""), scoped_token1, tag("\"")),
        scoped_token1,
    ))(input)
}

fn startuml(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    delimited(multispace0, tag("@startuml"), line_ending)(input)
}

fn enduml(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    delimited(space0, tag("@enduml"), multispace0)(input)
}

struct Connector;
fn connector(input: &str) -> IResult<&str, Connector, VerboseError<&str>> {
    let (input, _) = recognize(delimited(tag("-"), take_until(">"), tag(">")))(input)?;
    Ok((input, Connector))
}

fn description(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    let (input, _) = delimited(space0, tag(":"), space0)(input)?;
    let (input, a) = is_not("\n")(input)?;
    let result = if a.starts_with("\"\"") && a.ends_with("\"\"") && a.len() > 4 {
        &a[2..a.len() - 2]
    } else {
        a
    };

    Ok((input, result))
}

fn stereo(input: &str) -> IResult<&str, StateStereoType, VerboseError<&str>> {
    map(
        alt((
            delimited(tag("<<"), tag_no_case("start"), tag(">>")),
            delimited(tag("<<"), tag_no_case("choice"), tag(">>")),
            delimited(tag("<<"), tag_no_case("fork"), tag(">>")),
            delimited(tag("<<"), tag_no_case("join"), tag(">>")),
            delimited(tag("<<"), tag_no_case("end"), tag(">>")),
            delimited(tag("<<"), tag_no_case("sdlreceive"), tag(">>")),
            delimited(tag("<<"), tag_no_case("entryPoint"), tag(">>")),
            delimited(tag("<<"), tag_no_case("exitPoint"), tag(">>")),
            delimited(tag("<<"), tag_no_case("inputPin"), tag(">>")),
            delimited(tag("<<"), tag_no_case("outputPin"), tag(">>")),
            delimited(tag("<<"), tag_no_case("expansionInput"), tag(">>")),
            delimited(tag("<<"), tag_no_case("expansionOutput"), tag(">>")),
            delimited(tag("<<"), token1, tag(">>")),
        )),
        |t: &str| match t {
            "start" => StateStereoType::Start,
            "choice" => StateStereoType::Choice,
            "fork" => StateStereoType::Fork,
            "join" => StateStereoType::Join,
            "end" => StateStereoType::End,
            "sdlreceive" => StateStereoType::SdlReceive,
            "entryPoint" => StateStereoType::EntryPoint,
            "exitPoint" => StateStereoType::ExitPoint,
            "inputPin" => StateStereoType::InputPin,
            "outputPin" => StateStereoType::OutputPin,
            "expansionInput" => StateStereoType::ExpansionInput,
            "expansionOutput" => StateStereoType::ExpansionOutput,
            s => StateStereoType::Other(s.to_string()),
        },
    )(input)
}

fn inline_color(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    preceded(tag("#"), take_till(|c: char| c.is_whitespace()))(input)
}

fn inline_style(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    preceded(tag("##"), take_till(|c: char| c.is_whitespace()))(input)
}

fn lines(input: &str) -> IResult<&str, Vec<Line>, VerboseError<&str>> {
    delimited(
        terminated(tag("{"), multispace0),
        many0(line),
        preceded(space0, tag("}")),
    )(input)
}

#[derive(Clone, Debug)]
enum Either<L, R> {
    Left(L),
    Right(R),
}

#[derive(Clone, Debug)]
enum StateModifier {
    Start,
    End,
    History,
    DeepHistory,
}
#[derive(Clone, Debug)]
enum ScopedStateName {
    Plain {
        logical: VecDeque<String>,
    },
    Aliased {
        logical: VecDeque<String>,
        visual: String,
    },
}
#[derive(Clone, Debug)]
enum StateName {
    Plain { logical: String },
    Aliased { logical: String, visual: String },
}
#[derive(Clone, Debug)]
enum SpecialStateName {
    Start,
    End,
    History,
    DeepHistory,
}
#[derive(Clone, Debug)]
enum Item {
    ImplicitStateDeclaration {
        name: ScopedStateName,
        description: String,
    },
    KeywordStateDeclaration {
        name: ScopedStateName,
        stereotype: Option<StateStereoType>,
        description: Option<String>,
    },
    StateBlock {
        name: StateName,
        lines: Vec<Line>,
    },
    Transition {
        from_name: Either<
            (
                VecDeque<String>,
                Option<StateModifier>,
                Option<StateStereoType>,
            ),
            SpecialStateName,
        >,
        to_name: Either<
            (
                VecDeque<String>,
                Option<StateModifier>,
                Option<StateStereoType>,
            ),
            SpecialStateName,
        >,
        description: Option<String>,
    },
}

fn item(input: &str) -> IResult<&str, Item, VerboseError<&str>> {
    macro_rules! aliasiable_scoped_binding {
        () => {
            alt((
                map(
                    tuple((
                        terminated(
                            preceded(space0, quoted_string),
                            preceded(space1, tag_no_case("as")),
                        ),
                        alt((
                            map(preceded(space1, scoped_token1_maybe_quote), |s| {
                                s.into_iter().map(String::from).collect::<VecDeque<_>>()
                            }),
                            map(preceded(space1, token1_maybe_quote), |s| {
                                VecDeque::from([String::from(s)])
                            }),
                        )),
                    )),
                    |(v, l)| (Some(v), l),
                ),
                map(
                    tuple((
                        alt((
                            map(preceded(space0, scoped_token1_maybe_quote), |s| {
                                s.into_iter().map(String::from).collect::<VecDeque<_>>()
                            }),
                            map(preceded(space0, token1_maybe_quote), |s| {
                                VecDeque::from([String::from(s)])
                            }),
                        )),
                        preceded(
                            preceded(space1, tag_no_case("as")),
                            preceded(space1, quoted_string),
                        ),
                    )),
                    |(l, v)| (Some(v), l),
                ),
                map(
                    alt((
                        map(preceded(space0, scoped_token1_maybe_quote), |s| {
                            s.into_iter().map(String::from).collect::<VecDeque<_>>()
                        }),
                        map(preceded(space0, token1_maybe_quote), |s| {
                            VecDeque::from([String::from(s)])
                        }),
                    )),
                    |l| (None, l),
                ),
            ))
        };
    }
    let implicit_state = map(
        tuple((aliasiable_scoped_binding!(), map(description, String::from))),
        |((visual, logical), d)| Item::ImplicitStateDeclaration {
            name: if let Some(visual) = visual {
                ScopedStateName::Aliased { visual, logical }
            } else {
                ScopedStateName::Plain { logical }
            },
            description: d,
        },
    );

    let state = map(
        tuple((
            delimited(space0, tag_no_case("state"), peek(space1)),
            aliasiable_scoped_binding!(),
            opt(preceded(space1, stereo)),
            opt(preceded(space1, inline_color)),
            opt(preceded(space1, inline_style)),
            map(opt(description), |s| s.map(String::from)),
        )),
        |(_, (visual, logical), stereotype, _, _, description)| Item::KeywordStateDeclaration {
            name: if let Some(visual) = visual {
                ScopedStateName::Aliased { visual, logical }
            } else {
                ScopedStateName::Plain { logical }
            },
            stereotype,
            description,
        },
    );

    let state_block = map(
        tuple((
            delimited(space0, tag_no_case("state"), peek(space1)),
            alt((
                map(
                    tuple((
                        terminated(
                            preceded(space1, quoted_string),
                            preceded(space1, tag_no_case("as")),
                        ),
                        preceded(space1, token1),
                    )),
                    |(v, l)| (Some(v), l),
                ),
                map(
                    tuple((
                        preceded(space1, token1),
                        preceded(
                            preceded(space1, tag_no_case("as")),
                            preceded(space1, quoted_string),
                        ),
                    )),
                    |(l, v)| (Some(v), l),
                ),
                map(preceded(space1, token1), |l| (None, l)),
            )),
            opt(preceded(space1, inline_color)),
            opt(preceded(space1, inline_style)),
            preceded(space0, lines),
        )),
        |(_, (visual, logical), _, _, lines)| {
            let logical = logical.to_string();
            Item::StateBlock {
                name: if let Some(visual) = visual {
                    StateName::Aliased { logical, visual }
                } else {
                    StateName::Plain { logical }
                },
                lines,
            }
        },
    );

    let transition = map(
        tuple((
            alt((
                map(preceded(space0, tag("[*]")), |_| {
                    Either::Right(SpecialStateName::Start)
                }),
                map(preceded(space0, tag("[H*]")), |_| {
                    Either::Right(SpecialStateName::DeepHistory)
                }),
                map(preceded(space0, tag("[H]")), |_| {
                    Either::Right(SpecialStateName::History)
                }),
                map(
                    tuple((
                        alt((
                            map(preceded(space0, scoped_token1_maybe_quote), |t| {
                                t.into_iter()
                                    .map(|s| String::from(s))
                                    .collect::<VecDeque<_>>()
                            }),
                            map(preceded(space0, token1_maybe_quote), |t| {
                                VecDeque::from([String::from(t)])
                            }),
                        )),
                        opt(alt((
                            map(tag("[*]"), |_| StateModifier::Start),
                            map(tag("[H*]"), |_| StateModifier::DeepHistory),
                            map(tag("[H]"), |_| StateModifier::History),
                        ))),
                        opt(preceded(space1, stereo)),
                        opt(preceded(space1, inline_color)),
                        opt(preceded(space1, inline_style)),
                    )),
                    |(n, m, s, _, _)| Either::Left((n, m, s)),
                ),
            )),
            preceded(space0, connector),
            alt((
                map(preceded(space0, tag("[*]")), |_| {
                    Either::Right(SpecialStateName::End)
                }),
                map(preceded(space0, tag("[H*]")), |_| {
                    Either::Right(SpecialStateName::DeepHistory)
                }),
                map(preceded(space0, tag("[H]")), |_| {
                    Either::Right(SpecialStateName::History)
                }),
                map(
                    tuple((
                        alt((
                            map(preceded(space0, scoped_token1_maybe_quote), |t| {
                                t.into_iter()
                                    .map(|s| String::from(s))
                                    .collect::<VecDeque<_>>()
                            }),
                            map(preceded(space0, token1_maybe_quote), |t| {
                                VecDeque::from([String::from(t)])
                            }),
                        )),
                        opt(alt((
                            map(tag("[*]"), |_| StateModifier::End),
                            map(tag("[H*]"), |_| StateModifier::DeepHistory),
                            map(tag("[H]"), |_| StateModifier::History),
                        ))),
                        opt(preceded(space1, stereo)),
                        opt(preceded(space1, inline_color)),
                        opt(preceded(space1, inline_style)),
                    )),
                    |(n, m, s, _, _)| Either::Left((n, m, s)),
                ),
            )),
            map(opt(description), |s| s.map(String::from)),
        )),
        |(from_name, _, to_name, description)| Item::Transition {
            from_name,
            to_name,
            description,
        },
    );

    alt((state_block, state, transition, implicit_state))(input)
}

#[derive(Clone, Debug)]
enum Note {
    State {
        name: VecDeque<String>,
        content: Vec<String>,
    },
    LastTransition {
        content: Vec<String>,
    },
    Floating {
        content: String,
    },
}

fn note(input: &str) -> IResult<&str, Note, VerboseError<&str>> {
    alt((
        map(
            tuple((
                preceded(space0, tag_no_case("note")),
                preceded(space1, quoted_string),
                preceded(space1, tag_no_case("as")),
                preceded(space1, token1),
                opt(preceded(space1, inline_color)),
                opt(preceded(space1, style)),
            )),
            |(_, content, _, _, _, _)| Note::Floating { content },
        ),
        map(
            tuple((
                preceded(space0, tag_no_case("note")),
                preceded(
                    space1,
                    alt((
                        tag_no_case("left"),
                        tag_no_case("right"),
                        tag_no_case("bottom"),
                        tag_no_case("top"),
                    )),
                ),
                preceded(space1, tag_no_case("of")),
                map(preceded(space1, scoped_token1_maybe_quote), |x| {
                    x.into_iter()
                        .map(String::from)
                        .collect::<VecDeque<String>>()
                }),
                opt(preceded(space1, inline_color)),
                opt(preceded(space1, style)),
                multispace0,
                alt((
                    map(preceded(space0, description), |x| vec![x.to_string()]),
                    map(
                        many_till(
                            preceded(space0, many_till(anychar, line_ending)),
                            tuple((space0, tag_no_case("end"), space1, tag_no_case("note"))),
                        ),
                        |(lines, _): (Vec<(Vec<char>, _)>, _)| {
                            lines
                                .into_iter()
                                .map(|v| String::from_iter(v.0.into_iter()))
                                .collect::<Vec<_>>()
                        },
                    ),
                )),
            )),
            |(_, _, _, name, _, _, _, content)| Note::State { name, content },
        ),
        map(
            tuple((
                preceded(space0, tag_no_case("note")),
                preceded(space1, tag_no_case("on")),
                preceded(space1, tag_no_case("link")),
                opt(preceded(space1, inline_color)),
                opt(preceded(space1, style)),
                multispace0,
                map(
                    many_till(
                        preceded(space0, many_till(anychar, line_ending)),
                        tuple((space0, tag_no_case("end"), space1, tag_no_case("note"))),
                    ),
                    |(lines, _): (Vec<(Vec<char>, _)>, _)| {
                        lines
                            .into_iter()
                            .map(|v| String::from_iter(v.0.into_iter()))
                            .collect::<Vec<_>>()
                    },
                ),
            )),
            |(_, _, _, _, _, _, content)| Note::LastTransition { content },
        ),
    ))(input)
}

fn json_block(input: &str) -> IResult<&str, (), VerboseError<&str>> {
    map(
        tuple((
            preceded(space0, tag_no_case("json")),
            take_till(|c| c == '\n' || c == '{'),
            tag("{"),
            take_until("}"),
            tag("}"),
        )),
        |_| (),
    )(input)
}

fn skin_param(input: &str) -> IResult<&str, (), VerboseError<&str>> {
    map(
        tuple((
            preceded(space0, tag_no_case("skinparam")),
            alt((
                map(
                    tuple((
                        preceded(space0, take_till(|c| c == '\n' || c == '{')),
                        tag("{"),
                        take_until("}"),
                        tag("}"),
                    )),
                    |_| (),
                ),
                map(is_not("\n"), |_| ()),
            )),
        )),
        |_| (),
    )(input)
}

fn style(input: &str) -> IResult<&str, (), VerboseError<&str>> {
    map(
        tuple((
            preceded(space0, tag("<style>")),
            take_until("</style>"),
            tag("</style>"),
        )),
        |_| (),
    )(input)
}

fn directive(input: &str) -> IResult<&str, (), VerboseError<&str>> {
    preceded(
        space0,
        alt((
            map(tag_no_case("hide empty description"), |_| ()),
            map(
                tuple((
                    preceded(space0, tag_no_case("scale")),
                    preceded(space1, nom::character::complete::u32),
                    preceded(space1, tag_no_case("width")),
                )),
                |_| (),
            ),
        )),
    )(input)
}

fn concurrent_horizontal(input: &str) -> IResult<&str, (), VerboseError<&str>> {
    map(preceded(space0, tag("--")), |_| ())(input)
}

fn concurrent_vertical(input: &str) -> IResult<&str, (), VerboseError<&str>> {
    map(preceded(space0, tag("||")), |_| ())(input)
}

#[derive(Clone, Debug)]
enum Line {
    Style,
    JsonBlock,
    Directive,
    SkinParam,
    ConcurrentHorizontal,
    ConcurrentVertical,
    Space,
    Item(Item),
    Note(Note),
}
fn line(input: &str) -> IResult<&str, Line, VerboseError<&str>> {
    alt((
        map(
            terminated(space0::<&str, VerboseError<&str>>, line_ending),
            |_| Line::Space,
        ),
        map(
            delimited(space0, terminated(style, space0), line_ending),
            |_| Line::Style,
        ),
        map(
            delimited(space0, terminated(json_block, space0), line_ending),
            |_| Line::JsonBlock,
        ),
        map(
            delimited(space0, terminated(directive, space0), line_ending),
            |_| Line::Directive,
        ),
        map(
            delimited(space0, terminated(note, space0), line_ending),
            |n| Line::Note(n),
        ),
        map(
            delimited(space0, terminated(skin_param, space0), line_ending),
            |_| Line::SkinParam,
        ),
        map(
            delimited(
                space0,
                terminated(concurrent_horizontal, space0),
                line_ending,
            ),
            |_| Line::ConcurrentHorizontal,
        ),
        map(
            delimited(space0, terminated(concurrent_vertical, space0), line_ending),
            |_| Line::ConcurrentVertical,
        ),
        map(
            delimited(space0, terminated(item, space0), line_ending),
            |i| Line::Item(i),
        ),
    ))(input)
}

pub fn plantuml(input: &str) -> IResult<&str, Diagram, VerboseError<&str>> {
    let mut context = Context::new();
    let (input, lines) = delimited(startuml, many0(line), enduml)(input)?;
    for line in lines {
        if let Err(_) = context.process_line(line) {
            return nom::error::context("unrecognized syntax", fail)(input);
        };
    }
    let diagram = context.diagram();
    Ok((input, diagram))
}

pub fn human_readable_error<I, O, F>(f: F) -> impl FnOnce(I) -> anyhow::Result<(I, O)>
where
    F: FnOnce(I) -> IResult<I, O, VerboseError<I>>,
    I: Debug + Deref<Target = str> + Copy,
{
    |input| match f(input) {
        Err(nom::Err::Error(e)) => Err(anyhow!(convert_error(input, e))),
        Err(e) => Err(anyhow!("{}", e)),
        Ok(ok) => Ok(ok),
    }
}

enum Lexicon {
    StatePossessesConcurrentChildren(StateId),
    StateNote(StateId, Vec<String>),
    StateDescription(StateId, String),
    StateStereoType(StateId, StateStereoType),
    StateAlias(StateId, String),
    TransitionNote(TransitionId, Vec<String>),
    Transition(TransitionId),
    FloatingNote(String),
}

struct Context {
    frame_stack: Frames,
    scope: Scope,
    lex_log: Vec<Lexicon>,
}

impl Context {
    fn diagram(self) -> Diagram {
        let mut diagram: Diagram = Default::default();
        let frames_log = self.scope.flatten();
        for mut frames in frames_log {
            let frames = frames.make_contiguous();
            if frames.len() > 0 {
                let parent = StateId(VecDeque::new());
                let child = StateId([frames[0].clone()].into());
                diagram
                    .state_parent
                    .entry(child.clone())
                    .or_insert(parent.clone());
                diagram
                    .state_children
                    .entry(parent)
                    .or_default()
                    .insert(child);
                for i in 1..frames.len() {
                    let parent = StateId(frames[..=(i - 1)].iter().cloned().collect());
                    let child = StateId(frames[..=i].iter().cloned().collect());
                    diagram
                        .state_parent
                        .entry(child.clone())
                        .or_insert(parent.clone());
                    diagram
                        .state_children
                        .entry(parent)
                        .or_default()
                        .insert(child);
                }
            }
        }
        for entry in self.lex_log {
            match entry {
                Lexicon::StatePossessesConcurrentChildren(s) => {
                    diagram.state_children_are_concurrent.insert(s);
                }
                Lexicon::StateNote(s, n) => {
                    diagram.state_note.entry(s).or_default().extend(n);
                }
                Lexicon::StateDescription(s, n) => {
                    diagram.state_description.entry(s).or_default().push(n);
                }
                Lexicon::StateAlias(s, a) => {
                    diagram.state_alias.entry(s).or_insert(a);
                }
                Lexicon::StateStereoType(s, t) => {
                    diagram.state_stereotype.entry(s).or_insert(t);
                }
                Lexicon::TransitionNote(t, n) => {
                    diagram.transition_note.entry(t).or_default().extend(n);
                }
                Lexicon::Transition(t) => {
                    let a = t.0.clone();
                    let b = t.1.clone();
                    diagram
                        .state_transition_out
                        .entry(a.clone())
                        .or_default()
                        .insert(t.clone());
                    diagram
                        .state_transition_in
                        .entry(b.clone())
                        .or_default()
                        .insert(t.clone());
                    diagram
                        .transition_from
                        .entry(t.clone())
                        .or_insert(a.clone());
                    diagram.transition_to.entry(t.clone()).or_insert(b.clone());
                }
                Lexicon::FloatingNote(n) => {
                    diagram.note.push(n);
                }
            }
        }
        diagram
    }
    fn new() -> Self {
        Context {
            frame_stack: Frames {
                frames: VecDeque::new(),
            },
            scope: Default::default(),
            lex_log: vec![],
        }
    }

    fn process_line(&mut self, line: Line) -> anyhow::Result<()> {
        match line {
            Line::Style => Ok(()),
            Line::JsonBlock => Ok(()),
            Line::Directive => Ok(()),
            Line::SkinParam => Ok(()),
            Line::ConcurrentHorizontal => self.process_concurrent(),
            Line::ConcurrentVertical => self.process_concurrent(),
            Line::Space => Ok(()),
            Line::Item(item) => self.process_item(item),
            Line::Note(note) => self.process_note(note),
        }
    }

    fn process_concurrent(&mut self) -> anyhow::Result<()> {
        if self.frame_stack.frames.len() == 0 {
            return Ok(());
        }
        let mut frames = self.frame_stack.frames.clone();
        self.scope
            .resume_or_insert(&self.frame_stack.frames, &mut frames);
        self.lex_log
            .push(Lexicon::StatePossessesConcurrentChildren(StateId(frames)));
        Ok(())
    }

    fn process_item(&mut self, item: Item) -> anyhow::Result<()> {
        use Item::*;
        use ScopedStateName::*;
        match item {
            ImplicitStateDeclaration {
                name: Aliased { logical, visual },
                description,
            } => {
                let mut logical = logical
                    .into_iter()
                    .map(|name| Frame::State { name: name.clone() })
                    .collect();
                self.scope
                    .resume_or_insert(&self.frame_stack.frames, &mut logical);
                self.lex_log
                    .push(Lexicon::StateAlias(StateId(logical.clone()), visual));
                self.lex_log
                    .push(Lexicon::StateDescription(StateId(logical), description));
            }
            ImplicitStateDeclaration {
                name: Plain { logical },
                description,
            } => {
                let mut logical = logical
                    .into_iter()
                    .map(|name| Frame::State { name: name.clone() })
                    .collect();
                self.scope
                    .resume_or_insert(&self.frame_stack.frames, &mut logical);
                self.lex_log
                    .push(Lexicon::StateDescription(StateId(logical), description));
            }
            KeywordStateDeclaration {
                name: Aliased { logical, visual },
                description,
                stereotype,
            } => {
                let mut logical = logical
                    .into_iter()
                    .map(|name| Frame::State { name: name.clone() })
                    .collect();
                self.scope
                    .resume_or_insert(&self.frame_stack.frames, &mut logical);
                self.lex_log
                    .push(Lexicon::StateAlias(StateId(logical.clone()), visual));
                if let Some(description) = description {
                    self.lex_log
                        .push(Lexicon::StateDescription(StateId(logical), description));
                }
                match stereotype {
                    Some(StateStereoType::Start) => {}
                    Some(StateStereoType::End) => {}
                    Some(StateStereoType::Choice) => {}
                    Some(StateStereoType::Fork) => {}
                    Some(StateStereoType::Join) => {}
                    Some(StateStereoType::SdlReceive) => {}
                    Some(StateStereoType::EntryPoint) => {}
                    Some(StateStereoType::ExitPoint) => {}
                    Some(StateStereoType::InputPin) => {}
                    Some(StateStereoType::OutputPin) => {}
                    Some(StateStereoType::ExpansionInput) => {}
                    Some(StateStereoType::ExpansionOutput) => {}
                    Some(StateStereoType::Other(_)) => {}
                    None => {}
                }
            }
            KeywordStateDeclaration {
                name: Plain { logical },
                description,
                stereotype,
            } => {
                let mut logical = logical
                    .into_iter()
                    .map(|name| Frame::State { name: name.clone() })
                    .collect();
                self.scope
                    .resume_or_insert(&self.frame_stack.frames, &mut logical);
                if let Some(description) = description {
                    self.lex_log.push(Lexicon::StateDescription(
                        StateId(logical.clone()),
                        description,
                    ));
                }
                if let Some(s) = stereotype {
                    self.lex_log
                        .push(Lexicon::StateStereoType(StateId(logical), s))
                }
            }
            StateBlock {
                name: StateName::Aliased { logical, visual },
                lines,
            } => {
                let mut logical = [Frame::State { name: logical }].into();
                self.scope
                    .resume_or_insert(&self.frame_stack.frames, &mut logical);
                self.lex_log
                    .push(Lexicon::StateAlias(StateId(logical.clone()), visual));
                for line in lines {
                    self.process_line(line)?;
                }
            }
            StateBlock {
                name: StateName::Plain { logical },
                lines,
            } => {
                let oldframes = self.frame_stack.frames.clone();
                let mut logi = VecDeque::new();
                logi.push_back(Frame::State { name: logical });
                self.scope
                    .resume_or_insert(&self.frame_stack.frames, &mut logi);
                self.frame_stack.frames = logi;
                for line in lines {
                    self.process_line(line)?;
                }
                self.frame_stack.frames = oldframes;
            }
            Transition {
                from_name,
                to_name,
                description,
            } => {
                let (from_logical, from_stereotype) = match from_name {
                    Either::Left((logical, modifier, stereotype)) => {
                        let mut logical: VecDeque<_> = logical
                            .into_iter()
                            .map(|name| Frame::State { name: name.clone() })
                            .collect();
                        match modifier {
                            Some(StateModifier::Start) => logical.push_back(Frame::Start),
                            Some(StateModifier::End) => logical.push_back(Frame::End),
                            Some(StateModifier::History) => logical.push_back(Frame::History),
                            Some(StateModifier::DeepHistory) => {
                                logical.push_back(Frame::DeepHistory)
                            }
                            None => {}
                        }
                        self.scope
                            .resume_or_insert(&self.frame_stack.frames, &mut logical);
                        (logical, stereotype)
                    }
                    Either::Right(SpecialStateName::Start) => {
                        let mut logical = [Frame::Start].into();
                        self.scope
                            .resume_or_insert(&self.frame_stack.frames, &mut logical);
                        (logical, None)
                    }
                    Either::Right(SpecialStateName::End) => {
                        let mut logical = [Frame::End].into();
                        self.scope
                            .resume_or_insert(&self.frame_stack.frames, &mut logical);
                        (logical, None)
                    }
                    Either::Right(SpecialStateName::History) => {
                        let mut logical = [Frame::History].into();
                        self.scope
                            .resume_or_insert(&self.frame_stack.frames, &mut logical);
                        (logical, None)
                    }
                    Either::Right(SpecialStateName::DeepHistory) => {
                        let mut logical = [Frame::DeepHistory].into();
                        self.scope
                            .resume_or_insert(&self.frame_stack.frames, &mut logical);
                        (logical, None)
                    }
                };
                let (to_logical, to_stereotype) = match to_name {
                    Either::Left((logical, modifier, to_stereotype)) => {
                        let mut logical: VecDeque<_> = logical
                            .into_iter()
                            .map(|name| Frame::State { name: name.clone() })
                            .collect();
                        match modifier {
                            Some(StateModifier::Start) => logical.push_back(Frame::Start),
                            Some(StateModifier::End) => logical.push_back(Frame::End),
                            Some(StateModifier::History) => logical.push_back(Frame::History),
                            Some(StateModifier::DeepHistory) => {
                                logical.push_back(Frame::DeepHistory)
                            }
                            None => {}
                        }
                        self.scope
                            .resume_or_insert(&self.frame_stack.frames, &mut logical);
                        (logical, to_stereotype)
                    }
                    Either::Right(SpecialStateName::Start) => {
                        let mut logical = [Frame::Start].into();
                        self.scope
                            .resume_or_insert(&self.frame_stack.frames, &mut logical);
                        (logical, None)
                    }
                    Either::Right(SpecialStateName::End) => {
                        let mut logical = [Frame::End].into();
                        self.scope
                            .resume_or_insert(&self.frame_stack.frames, &mut logical);
                        (logical, None)
                    }
                    Either::Right(SpecialStateName::History) => {
                        let mut logical = [Frame::History].into();
                        self.scope
                            .resume_or_insert(&self.frame_stack.frames, &mut logical);
                        (logical, None)
                    }
                    Either::Right(SpecialStateName::DeepHistory) => {
                        let mut logical = [Frame::DeepHistory].into();
                        self.scope
                            .resume_or_insert(&self.frame_stack.frames, &mut logical);
                        (logical, None)
                    }
                };
                let from_state = StateId(from_logical);
                let to_state = StateId(to_logical);
                if let Some(s) = from_stereotype {
                    self.lex_log
                        .push(Lexicon::StateStereoType(from_state.clone(), s))
                }
                if let Some(s) = to_stereotype {
                    self.lex_log
                        .push(Lexicon::StateStereoType(to_state.clone(), s))
                }
                let transition = TransitionId(from_state, to_state, description);
                self.lex_log.push(Lexicon::Transition(transition.clone()));
            }
        }
        Ok(())
    }

    fn process_note(&mut self, note: Note) -> anyhow::Result<()> {
        match note {
            Note::State { name, content } => {
                let mut logical = name
                    .into_iter()
                    .map(|name| Frame::State { name: name.clone() })
                    .collect();
                self.scope
                    .resume_or_insert(&self.frame_stack.frames, &mut logical);
                self.lex_log
                    .push(Lexicon::StateNote(StateId(logical), content))
            }
            Note::LastTransition { content } => {
                if let Some(t) = self.lex_log.iter().rev().find_map(|e| {
                    if let Lexicon::Transition(t) = e {
                        Some(t)
                    } else {
                        None
                    }
                }) {
                    self.lex_log
                        .push(Lexicon::TransitionNote(t.clone(), content));
                }
            }
            Note::Floating { content } => self.lex_log.push(Lexicon::FloatingNote(content)),
        }
        Ok(())
    }
}
