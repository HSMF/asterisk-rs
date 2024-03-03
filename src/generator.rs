use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fmt::Display,
};

use itertools::Itertools;

use crate::{
    grammar::{Grammar, Token},
    string_pool::{Id, Pool},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Production(Id, Vec<Token>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct State {
    pub(crate) rule: Id,
    pub(crate) before: Vec<Token>,
    pub(crate) after: Vec<Token>,
    pub(crate) lookahead: BTreeSet<Token>,
}

impl State {
    pub fn new(rule: Id, productions: Vec<Token>) -> Self {
        Self {
            rule,
            before: Vec::new(),
            after: productions,
            lookahead: BTreeSet::new(),
        }
    }

    fn display<'a>(&'a self, pool: &'a Pool) -> StateDisplay<'a> {
        StateDisplay { state: self, pool }
    }
}

pub struct StateDisplay<'a> {
    state: &'a State,
    pool: &'a Pool,
}

impl Display for StateDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pool = self.pool;
        write!(f, "{} -> ", pool.get(self.state.rule))?;
        for tok in &self.state.before {
            write!(f, "{} ", tok.display(pool))?;
        }
        write!(f, ".")?;
        for tok in &self.state.after {
            write!(f, "{} ", tok.display(pool))?;
        }

        write!(f, " {{ ")?;
        for tok in &self.state.lookahead {
            write!(f, "{} ", tok.display(pool))?;
        }

        write!(f, "}}")?;

        Ok(())
    }
}

pub type Uid = usize;

#[derive(Debug, Clone)]
pub struct Graph(pub(crate) HashMap<Uid, (HashSet<State>, HashMap<Uid, Token>)>);

struct GenUid(usize);
impl GenUid {
    fn next(&mut self) -> usize {
        self.0 += 1;
        self.0
    }
}

fn closure(grammar: &Grammar, mut states: HashSet<State>) -> HashSet<State> {
    loop {
        let mut new_state = states.clone();
        let mut num_added = 0;
        for state in &states {
            let [c, delta @ ..] = state.after.as_slice() else {
                continue;
            };

            let mut lookahead = grammar.first(delta);
            let Token::NonTerm(c) = c else {
                continue;
            };

            if lookahead.contains(&Token::Empty) {
                lookahead.remove(&Token::Empty);
                for &tok in &state.lookahead {
                    lookahead.insert(tok);
                }
            }

            let added = grammar.initial(*c).into_iter().map(move |x| State {
                lookahead: lookahead.clone(),
                ..x
            });
            for i in added {
                let could_add = new_state.insert(i);
                num_added += if could_add { 1 } else { 0 };
            }
        }

        if num_added == 0 {
            return new_state;
        }

        states = new_state;
    }
}

fn out_edges<'a, 'b>(states: &'b HashSet<State>) -> impl IntoIterator<Item = Token> + 'a
where
    'b: 'a,
{
    states
        .iter()
        .filter_map(|x| x.after.split_first().map(|(&x, _)| x))
        .collect::<HashSet<_>>()
}

fn advance_states<'a>(edge: Token, states: impl Iterator<Item = &'a State>) -> HashSet<State> {
    states
        .filter_map(|x| {
            let (hd, tl) = x.after.split_first()?;
            if hd != &edge {
                return None;
            }

            let mut x = x.to_owned();
            x.after = tl.to_owned();
            x.before.push(edge);
            Some(x)
        })
        .collect()
}

impl Graph {
    pub fn make(grammar: &Grammar, states: HashSet<State>) -> Graph {
        fn inner(
            g: &mut Graph,
            grammar: &Grammar,
            uid: &mut GenUid,
            states: HashSet<State>,
        ) -> usize {
            let states = closure(grammar, states);
            // state already exists in the graph, just go to the state
            if let Some((&key, _)) = g.0.iter().find(|(_, (x, _))| *x == states) {
                return key;
            }

            let id = uid.next();

            g.0.insert(id, (states.clone(), HashMap::new()));
            for edge in out_edges(&states) {
                let advanced = advance_states(edge, states.iter());
                let next = inner(g, grammar, uid, advanced);
                let next_entry = g.0.entry(id);
                // println!(
                //     "adding {next} to {id}. reason: {}",
                //     edge.display(grammar.pool())
                // );
                next_entry
                    .and_modify(|x| {
                        x.1.insert(next, edge);
                    })
                    .or_insert_with(|| (states.clone(), HashMap::new()));
            }

            id
        }

        let mut graph = Graph(HashMap::new());
        let mut gen_uid = GenUid(0);
        inner(&mut graph, grammar, &mut gen_uid, states);

        graph
    }

    pub fn print<'a>(&'a self, pool: &'a Pool) -> GraphPrinter<'a> {
        GraphPrinter { graph: self, pool }
    }
}

pub struct GraphPrinter<'a> {
    graph: &'a Graph,
    pool: &'a Pool,
}

impl Display for GraphPrinter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "digraph {{")?;
        for (node, (states, neighbors)) in &self.graph.0 {
            write!(
                f,
                r##"node{node} [fillcolor="lightblue",style=filled,shape=box,labeljust=l,class="node",id="node{node}",label="node{node}\l"##
            )?;
            for state in states.iter().sorted_by(|a, b| {
                match self.pool.get(a.rule).cmp(self.pool.get(b.rule)) {
                    std::cmp::Ordering::Equal => a.cmp(b),
                    x => x,
                }
            }) {
                write!(f, "{}\\l", state.display(self.pool))?;
            }
            writeln!(f, r#""];"#)?;

            for (&ni, neighbor) in neighbors {
                write!(
                    f,
                    r#"node{node} -> node{ni} [label="{}",class="from-node{node} to-node{ni}"]"#,
                    neighbor.display(self.pool)
                )?;
            }
        }

        writeln!(f, "}}")?;
        Ok(())
    }
}
