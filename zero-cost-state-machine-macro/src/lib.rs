extern crate proc_macro;
use anyhow::bail;
use heck::{ToSnakeCase, ToUpperCamelCase};
use itertools::Either::{Left, Right};
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::VecDeque;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Read;
use std::iter;
use syn::LitStr;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, Ident, Result};
use zero_cost_state_machine_puml::{frames, TransitionId};
use zero_cost_state_machine_puml::{state_id, Frame, StateId};
use zero_cost_state_machine_puml::{Diagram, Frames};

#[cfg(test)]
mod test;

#[derive(Debug)]
struct FileName {
    filename: String,
}

impl Parse for FileName {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit_file: syn::LitStr = input.parse()?;
        Ok(Self {
            filename: lit_file.value(),
        })
    }
}

struct MacroInput {
    contents: LitStr,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let contents: LitStr = input.parse()?;
        Ok(MacroInput { contents })
    }
}

#[derive(Default, PartialEq, Debug)]
struct Aux<'a> {
    pub transition_to_start_redirection: BTreeMap<&'a TransitionId, &'a StateId>,
    pub transition_from_end_redirection: BTreeMap<&'a TransitionId, &'a StateId>,
    pub child_node_canonical_name: BTreeMap<&'a StateId, String>,
    pub edge_canonical_name: BTreeMap<&'a TransitionId, Option<String>>,
    pub relative_canonical_name:
        BTreeMap<&'a TransitionId, (Vec<String>, usize, Vec<String>, usize)>,
}

impl<'a> Aux<'a> {
    fn human_readable_name(state: &StateId) -> anyhow::Result<Option<String>> {
        let parent_of_origin: anyhow::Result<String> = state
            .0
            .range(0..state.0.len().saturating_sub(1))
            .map(|f| match f {
                Frame::Start => bail!("the special state Start cannot have children"),
                Frame::End => bail!("the special state End cannot have children"),
                Frame::History => bail!("the special state History cannot have children"),
                Frame::DeepHistory => bail!("the special state DeepHistory cannot have children"),
                Frame::State { name } => Ok(format!("{}.", name)),
            })
            .collect();
        let parent_of_origin = parent_of_origin?;
        let origin = match state.0.iter().last() {
            Some(Frame::Start) => anyhow::Ok("[*]".into()),
            Some(Frame::End) => anyhow::Ok("[*]".into()),
            Some(Frame::History) => anyhow::Ok("[H]".into()),
            Some(Frame::DeepHistory) => anyhow::Ok("[H*]".into()),
            Some(Frame::State { name }) => anyhow::Ok(name.clone()),
            None => return Ok(None),
        }?;
        Ok(Some(format!("{}{}", parent_of_origin, origin)))
    }
    fn transition_from_end_redirection(
        diagram: &Diagram,
    ) -> anyhow::Result<BTreeMap<&TransitionId, &StateId>> {
        let mut transition_from_end_redirection: BTreeMap<&TransitionId, &StateId> =
            BTreeMap::new();
        for (state, children) in diagram.state_children.iter() {
            let s = Self::human_readable_name(state)?;
            let s = s
                .map(|s| format!("state {}", s))
                .unwrap_or(format!("the special state Root"));
            let end_state = children.iter().find(|s| match s.0.iter().last() {
                Some(Frame::End) => true,
                _ => false,
            });
            if end_state.is_none() {
                if diagram.state_transition_out.contains_key(state) {
                    bail!("{} must contain an End state", s);
                }
            }
            for transition in diagram
                .state_transition_out
                .get(state)
                .iter()
                .flat_map(|s| s.iter())
            {
                if let Some(end_state) = end_state {
                    transition_from_end_redirection.insert(transition, end_state);
                }
            }
        }
        Ok(transition_from_end_redirection)
    }
    fn transition_to_start_redirection(
        diagram: &Diagram,
    ) -> anyhow::Result<BTreeMap<&TransitionId, &StateId>> {
        let mut transition_to_start_redirection: BTreeMap<&TransitionId, &StateId> =
            BTreeMap::new();
        for (state, children) in diagram.state_children.iter() {
            let s = Self::human_readable_name(state)?;
            let s = s
                .map(|s| format!("state {}", s))
                .unwrap_or(format!("the special state Root"));
            let start_state = children.iter().find(|s| match s.0.iter().last() {
                Some(Frame::Start) => true,
                _ => false,
            });
            if start_state.is_none() {
                if diagram.state_transition_in.contains_key(state) {
                    bail!("{} must contain Start state", s);
                }
            }
            for transition in diagram
                .state_transition_in
                .get(state)
                .iter()
                .flat_map(|s| s.iter())
            {
                if let Some(start_state) = start_state {
                    transition_to_start_redirection.insert(transition, start_state);
                }
            }
        }
        Ok(transition_to_start_redirection)
    }
    fn child_node_canonical_name(diagram: &Diagram) -> anyhow::Result<BTreeMap<&StateId, String>> {
        let mut child_node_canonical_name: BTreeMap<&StateId, String> = BTreeMap::new();
        for (state, children) in diagram.state_children.iter() {
            let s = Self::human_readable_name(state)?;
            let s = s
                .map(|s| format!("state {}", s))
                .unwrap_or(format!("the special state Root"));
            for child in children {
                match child.0.iter().last() {
                    Some(Frame::State { name }) => {
                        let converted = name.to_upper_camel_case();
                        if let Some(n) = child_node_canonical_name.get(child) {
                            if n == &converted {
                                bail!("{} contains multiple child states which when converted to upper camel case are {}", s, converted);
                            }
                        }
                        child_node_canonical_name.insert(child, name.to_upper_camel_case());
                    }
                    Some(Frame::Start) => {
                        child_node_canonical_name.insert(child, "Start".into());
                    }
                    Some(Frame::End) => {
                        child_node_canonical_name.insert(child, "End".into());
                    }
                    Some(Frame::History) => {
                        child_node_canonical_name.insert(child, "History".into());
                    }
                    Some(Frame::DeepHistory) => {
                        child_node_canonical_name.insert(child, "DeepHistory".into());
                    }
                    None => {}
                }
            }
        }
        Ok(child_node_canonical_name)
    }
    fn edge_canonical_name(
        diagram: &Diagram,
    ) -> anyhow::Result<BTreeMap<&TransitionId, Option<String>>> {
        let mut edge_canonical_name: BTreeMap<&TransitionId, Option<String>> = BTreeMap::new();
        for (state, edges) in diagram.state_transition_out.iter() {
            let s = Self::human_readable_name(state)?;
            let s = if let Some(s) = s {
                s
            } else {
                bail!("no transition can lead out of the special state Root");
            };
            for edge in edges {
                let converted = edge.2.as_ref().map(|e| e.to_upper_camel_case());
                match (converted, edge_canonical_name.get(edge)) {
                    (Some(a), Some(Some(b))) => {
                        if &a == b {
                            bail!("multiple transitions are exiting from state {} which when converted to upper camel case are {}", s, a);
                        }
                    }
                    (Some(a), Some(None)) => {
                        edge_canonical_name.insert(edge, Some(a));
                    }
                    (Some(a), None) => {
                        edge_canonical_name.insert(edge, Some(a));
                    }
                    (None, Some(Some(_))) => {
                        edge_canonical_name.insert(edge, None);
                    }
                    (None, Some(None)) => {}
                    (None, None) => {
                        edge_canonical_name.insert(edge, None);
                    }
                }
            }
        }
        Ok(edge_canonical_name)
    }
    fn relative_canonical_name(
        diagram: &'a Diagram,
        transition_from_end_redirection: &BTreeMap<&TransitionId, &StateId>,
        transition_to_start_redirection: &BTreeMap<&TransitionId, &StateId>,
    ) -> anyhow::Result<BTreeMap<&'a TransitionId, (Vec<String>, usize, Vec<String>, usize)>> {
        let mut relative_canonical_name: BTreeMap<
            &TransitionId,
            (Vec<String>, usize, Vec<String>, usize),
        > = BTreeMap::new();
        for (_state, edges) in diagram.state_transition_out.iter() {
            for edge in edges {
                let from_node = &edge.0;
                let from_node = transition_from_end_redirection
                    .get(edge)
                    .unwrap_or(&from_node);
                let from_node: Vec<_> = from_node.0.iter().collect();
                let to_node = &edge.1;
                let to_node = transition_to_start_redirection
                    .get(edge)
                    .unwrap_or(&to_node);
                let to_node: Vec<_> = to_node.0.iter().collect();

                let mut i = from_node.len();
                let mut j = to_node.len();
                loop {
                    if &from_node[..i] == &to_node[..j] {
                        break;
                    }
                    if i < j {
                        j -= 1;
                    } else if j < i {
                        i -= 1;
                    } else {
                        i -= 1;
                        j -= 1;
                    }
                }
                let origin_ascent = from_node.len() - i;
                let target_ascent = to_node.len() - j;
                let relative_ascent = origin_ascent.saturating_sub(target_ascent);

                let source_frames = from_node
                    .iter()
                    .last()
                    .into_iter()
                    .flat_map(|f| match f {
                        Frame::Start => ["node".into(), "Start".into()],
                        Frame::End => ["node".into(), "End".into()],
                        Frame::History => ["node".into(), "History".into()],
                        Frame::DeepHistory => ["node".into(), "DeepHistory".into()],
                        Frame::State { name } => ["node".into(), name.to_upper_camel_case()],
                    })
                    .collect();

                // stay on the same level when we have no relative ascent
                let supers = (0..relative_ascent).map(|_| "super".into());

                // descend through modules when target ascent is greater than origin ascent
                let modules = to_node[relative_ascent
                    ..relative_ascent + target_ascent.saturating_sub(origin_ascent)]
                    .into_iter()
                    .filter_map(|f| match f {
                        Frame::Start => None,
                        Frame::End => None,
                        Frame::History => None,
                        Frame::DeepHistory => None,
                        Frame::State { name } => Some(name.to_snake_case()),
                    });

                // final addressing
                let t = to_node.iter().last().into_iter().flat_map(|f| match f {
                    Frame::Start => ["node".into(), "Start".into()],
                    Frame::End => ["node".into(), "End".into()],
                    Frame::History => ["node".into(), "History".into()],
                    Frame::DeepHistory => ["node".into(), "DeepHistory".into()],
                    Frame::State { name } => ["node".into(), name.to_upper_camel_case()],
                });

                let target_frames = supers.chain(modules).chain(t).collect();

                relative_canonical_name.insert(
                    edge,
                    (source_frames, from_node.len(), target_frames, to_node.len()),
                );
            }
        }
        Ok(relative_canonical_name)
    }
    fn new(diagram: &'a Diagram) -> anyhow::Result<Self> {
        let child_node_canonical_name = Self::child_node_canonical_name(&diagram)?;
        let edge_canonical_name = Self::edge_canonical_name(&diagram)?;
        let transition_from_end_redirection = Self::transition_from_end_redirection(&diagram)?;
        let transition_to_start_redirection = Self::transition_to_start_redirection(&diagram)?;
        let relative_canonical_name = Self::relative_canonical_name(
            &diagram,
            &transition_from_end_redirection,
            &transition_to_start_redirection,
        )?;
        Ok(Aux {
            transition_to_start_redirection,
            transition_from_end_redirection,
            child_node_canonical_name,
            edge_canonical_name,
            relative_canonical_name,
        })
    }
}

fn module(
    diagram: &Diagram,
    aux @ Aux {
        child_node_canonical_name,
        edge_canonical_name,
        relative_canonical_name,
        ..
    }: &Aux,
    root: &StateId,
) -> TokenStream {
    let depth = root.0.len();
    let def = BTreeSet::new();
    let child_nodes = if let Some(s) = diagram.state_children.get(root) {
        s
    } else {
        &def
    };
    let exits = &child_nodes
        .iter()
        .filter(|s| diagram.state_children.contains_key(s))
        .map(|s| diagram.state_transition_out.get(s))
        .fold(BTreeSet::new(), |mut acc, s| {
            acc.extend(s.into_iter().flatten().cloned());
            acc
        });
    let child_edges: BTreeSet<_> = child_nodes
        .iter()
        .flat_map(|s| {
            diagram
                .state_transition_out
                .get(s)
                .into_iter()
                .flat_map(|t| t.difference(exits))
                .chain(
                    diagram
                        .state_transition_out
                        .get(root)
                        .into_iter()
                        .flat_map(|s| s.into_iter()),
                )
        })
        .collect();

    let child_nodes: BTreeSet<_> = child_nodes
        .iter()
        .filter_map(|s| child_node_canonical_name.get(s))
        .collect();
    let child_nodes = child_nodes
        .into_iter()
        .map(|s| Ident::new(s, Span::call_site()))
        .map(|s| {
            quote! {
                pub struct #s;
            }
        });
    let child_edges: BTreeSet<_> = child_edges
        .iter()
        .filter_map(|t| edge_canonical_name.get(t))
        .collect();
    let child_edges = child_edges
        .into_iter()
        .flat_map(|s| s.as_ref().map(|s| Ident::new(s, Span::call_site())))
        .map(|s| {
            quote! {
                pub struct #s;
            }
        });
    let nodemod = quote! {
        pub mod node {
            #(#child_nodes)*
        }
    };
    let edgemod = quote! {
        pub mod edge {
            #(#child_edges)*
        }
    };
    let node_paths = (0..depth)
        .map(|i| {
            (
                Ident::new(format!("node{}", i).as_str(), Span::call_site()),
                Ident::new(format!("N{}", i).as_str(), Span::call_site()),
                Ident::new(format!("edge{}", i).as_str(), Span::call_site()),
                Ident::new(format!("E{}", i).as_str(), Span::call_site()),
            )
        })
        .map(|(node, s, edge, p)| {
            quote! {
                pub #node: #s,
                pub #edge: #p,
            }
        });

    let node_edge_type_params: Vec<_> = (0..depth)
        .flat_map(|i| {
            [
                Ident::new(format!("N{}", i).as_str(), Span::call_site()),
                Ident::new(format!("E{}", i).as_str(), Span::call_site()),
            ]
        })
        .map(|s| {
            quote! {
                #s
            }
        })
        .collect();
    let node_edge_type_params = &node_edge_type_params;

    let v = &quote! {S};
    let state_struct_node_edge_type_params = node_edge_type_params.iter().chain(iter::once(v));

    let state_struct = quote! {
        pub struct State<#(#state_struct_node_edge_type_params),*> {
            #(#node_paths)*
            pub node: S
        }
    };

    let child_transitions = diagram
        .state_children
        .get(root)
        .into_iter()
        .flat_map(|s| s.iter())
        .filter(|s| !diagram.state_children.contains_key(s))
        .flat_map(|s| {
            diagram
                .state_transition_out
                .get(s)
                .into_iter()
                .flat_map(|s| s.iter())
        })
        .filter(|t| {
            edge_canonical_name.contains_key(t) &&
                child_node_canonical_name.contains_key(&t.0) &&
                relative_canonical_name.contains_key(t)
        })
        .chain(
            diagram.state_transition_out.get(root)
                .into_iter()
                .flat_map(|t| t.into_iter())
        )
        .map(|t| {
            let transition = &if let Some(s) = &edge_canonical_name[t] {
                let transition = Ident::new(s, Span::call_site());
                quote! {
                    edge::#transition
                }
            } else {
                quote! {()}
            };
            let state = &String::from("State");
            let (from_node, origin_depth, to_node, target_depth) = &relative_canonical_name[t];
            let ascent_to_target = to_node.iter().take_while(|s| s.as_str() != "node").chain(iter::once(state));
            let ascent_to_target = ascent_to_target.map(|s| Ident::new(s, Span::call_site()));
            let ascent_to_target = quote! { #(#ascent_to_target)::* };
            
            let from_node = from_node.iter().map(|s| Ident::new(s, Span::call_site()));
            let from_node = &quote! { #(#from_node)::* };
            let to_node = to_node.iter().map(|s| Ident::new(s, Span::call_site()));
            let to_node = quote! { #(#to_node)::* };

            let nfn = &quote! {#from_node};
            let state_origin_node_edge_type_params = node_edge_type_params
                .iter()
                .chain(
                    iter::once(nfn)
                );
                
            let tn = &quote! {#to_node};
            let state_destination_node_edge_type_params = node_edge_type_params
                .iter()
                .take((target_depth.saturating_sub(1))*2)
                .chain({
                    let it = if target_depth > origin_depth {
                        Some([nfn, transition])
                    } else {
                        None
                    };
                    let it = it.into_iter();
                    it.flatten()
                })
                .chain(
                    iter::once(tn)
                );
                
            let targ = &quote! {
                 #ascent_to_target<#(#state_destination_node_edge_type_params),*>
            };
            
            let bindings = {
                if target_depth.saturating_sub(*origin_depth) > 0 {
                    Left(
                        (0..target_depth.saturating_sub(2)).map(|i| {
                            let node_field = &Ident::new(format!("node{}", i).as_str(), Span::call_site());
                            let edge_field = &Ident::new(format!("edge{}", i).as_str(), Span::call_site());
                            quote! {
                                #node_field: self.#node_field,
                                #edge_field: self.#edge_field,
                            }
                        }).chain(
                            (target_depth.saturating_sub(2)..target_depth.saturating_sub(1)).map(|i| {
                                let node_field = &Ident::new(format!("node{}", i).as_str(), Span::call_site());
                                let edge_field = &Ident::new(format!("edge{}", i).as_str(), Span::call_site());
                                quote! {
                                    #node_field: #from_node,
                                    #edge_field: path,
                                }
                            })
                        )
                    )
                } else if target_depth.saturating_sub(*origin_depth) == 0 {
                    Right(
                        Left(
                            (0..target_depth.saturating_sub(1)).map(|i| {
                                let node_field = &Ident::new(format!("node{}", i).as_str(), Span::call_site());
                                let edge_field = &Ident::new(format!("edge{}", i).as_str(), Span::call_site());
                                quote! {
                                    #node_field: self.#node_field,
                                    #edge_field: self.#edge_field,
                                }
                            })
                        )
                    )
                } else {
                    Right(
                        Right(
                            (0..target_depth.saturating_sub(1)).map(|i| {
                                let node_field = &Ident::new(format!("node{}", i).as_str(), Span::call_site());
                                let edge_field = &Ident::new(format!("edge{}", i).as_str(), Span::call_site());
                                quote! {
                                    #node_field: self.#node_field,
                                    #edge_field: self.#edge_field,
                                }
                            })
                        )
                    )
                }
            }.chain(iter::once({
                quote! {
                    node: #to_node
                }
            }));

            quote! {
                impl<#(#node_edge_type_params),*> zero_cost_state_machine::Switch<#transition> for State<#(#state_origin_node_edge_type_params),*> {
                    type Target = #targ;
                    fn transition(self, path: #transition) -> Self::Target {
                        Self::Target {
                            #(#bindings)*
                        }
                    }
                }
            }
        });
    let mods = diagram
        .state_children
        .get(root)
        .into_iter()
        .flat_map(|s| s.iter())
        .filter(|s| diagram.state_children.contains_key(*s))
        .map(|s| {
            let m = module(diagram, aux, s);
            if let Some(Frame::State { name }) = s.0.iter().last() {
                let name = Ident::new(&*name.to_snake_case(), Span::call_site());
                quote! {
                    pub mod #name {
                        #m
                    }
                }
            } else {
                quote! {}
            }
        });
    quote! {
        #nodemod
        #edgemod
        #state_struct
        #(#child_transitions)*
        #(#mods)*
    }
}

#[proc_macro]
pub fn statemachine_from_puml(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let MacroInput { contents } = parse_macro_input!(input as MacroInput);
    let contents = contents.value();

    let (_, diagram) = match zero_cost_state_machine_puml::human_readable_error(
        zero_cost_state_machine_puml::plantuml,
    )(&*contents)
    {
        Ok((input, diagram)) => (input, diagram),
        Err(e) => {
            let error_message = e.to_string();
            return quote! {
                compile_error!(#error_message);
            }
            .into();
        }
    };

    let aux = &match Aux::new(&diagram) {
        Ok(a) => a,
        Err(e) => {
            let error_message = e.to_string();
            return quote! {
                compile_error!(#error_message);
            }
            .into();
        }
    };

    return module(&diagram, aux, &state_id![]).into();
}

#[proc_macro]
pub fn statemachine_from_puml_file(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as FileName);
    let cwd = std::env::current_dir().unwrap();
    let file_path = cwd.join(&input.filename);
    let file_path_str = file_path.display().to_string();
    let mut file = File::open(file_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let (_, _) = match zero_cost_state_machine_puml::human_readable_error(
        zero_cost_state_machine_puml::plantuml,
    )(&*contents)
    {
        Ok((input, diagram)) => (input, diagram),
        Err(e) => {
            let error_message = e.to_string();
            return quote! {
                compile_error!(#error_message);
            }
            .into();
        }
    };

    return quote! {
        const _this_triggers_rebuild_on_change: &'static str = include_str!(#file_path_str);
        pub struct Machine {
        }
    }
    .into();
}
