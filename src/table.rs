use std::{collections::HashMap, fmt::Display};

use either::Either;

use crate::{
    generator::{Graph, Uid},
    grammar::Token,
    string_pool::{Id, Pool},
};

#[derive(Debug, Clone)]
pub struct Conflict {
    pub token: Token,
    pub either: Action,
    pub or: Action,
    pub state: Uid,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Reduce(Id, Vec<Token>),
    Shift(Uid),
}

pub struct ActionDisplay<'a> {
    action: &'a Action,
    pool: &'a Pool,
}

impl Action {
    pub fn display<'a>(&'a self, pool: &'a Pool) -> ActionDisplay<'a> {
        ActionDisplay { action: self, pool }
    }
}

impl Display for ActionDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.action {
            Action::Reduce(r, expansion) => {
                write!(f, "{} ->", self.pool.get(*r))?;
                for e in expansion {
                    write!(f, " {}", e.display(self.pool))?;
                }
                Ok(())
            }
            Action::Shift(x) => write!(f, "s{}", x),
        }
    }
}

#[derive(Debug)]
pub struct TableEntry {
    pub(crate) actions: HashMap<Token, Action>,
    pub(crate) gotos: HashMap<Id, Uid>,
}

#[derive(Debug)]
pub struct Table(pub(crate) HashMap<Uid, TableEntry>);

impl Table {
    pub fn from_graph(graph: &Graph) -> Result<Self, Conflict> {
        let mut table = HashMap::new();

        for (&state_id, (states, neighbors)) in &graph.0 {
            let shifts = neighbors.iter().filter_map(|(a, &b)| {
                b.term()
                    .map(|b| (Token::Term(b), Action::Shift(*a)))
                    .or(b.eof().map(|_| (Token::Eof, Action::Shift(*a))))
            });

            let gotos = neighbors
                .iter()
                .filter_map(|(a, &b)| b.non_term().map(|b| (b, *a)));

            let reductions_iter = states.iter().filter(|x| x.after.is_empty()).flat_map(|x| {
                let reduce = Action::Reduce(x.rule, x.before.clone());
                if x.lookahead.is_empty() {
                    return Either::Left(std::iter::once((Token::Eof, reduce)));
                }
                Either::Right(
                    x.lookahead
                        .clone()
                        .into_iter()
                        .map(move |l| (l, reduce.clone())),
                )
            });

            let mut reductions: HashMap<Token, Action> = HashMap::new();
            for reduction in reductions_iter {
                // if let Some(conflict) = reductions.get(&reduction.0) {
                //     return Err(Conflict {
                //         token: reduction.0,
                //         either: reduction.1,
                //         or: conflict.clone(),
                //         state: state_id,
                //     });
                // }
                reductions.insert(reduction.0, reduction.1.clone());
            }

            for (k, shift) in shifts {
                if let Some(conflict) = reductions.get(&k) {
                    return Err(Conflict {
                        either: shift,
                        token: k,
                        or: conflict.clone(),
                        state: state_id,
                    });
                }
                reductions.insert(k, shift);
            }

            table.insert(
                state_id,
                TableEntry {
                    actions: reductions,
                    gotos: gotos.collect(),
                },
            );
        }

        Ok(Table(table))
    }
}
