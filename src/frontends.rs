use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Formatter, Result},
};

use itertools::Itertools;

use crate::{
    generator::Uid,
    grammar::{Grammar, Token},
    string_pool::Id,
    table::{Action, Table},
};

pub struct Ctx<'a> {
    grammar: &'a Grammar,
}

pub mod ocaml;
pub mod rust;

pub trait Visitor {
    /// This function is called before the table is "entered", i.e. at the very beginning
    /// It is only called once per writing process.
    fn before_enter(&self, ctx: &Ctx, f: &mut Formatter, all_states: &[Uid]) -> Result;
    /// This function is called after the table is "left", i.e. at the very end
    /// It is only called once per writing process.
    fn after_leave(&self, ctx: &Ctx, f: &mut Formatter, all_states: &[Uid]) -> Result;

    fn begin_parse_loop(&self, ctx: &Ctx, f: &mut Formatter) -> Result;
    fn end_parse_loop(&self, ctx: &Ctx, f: &mut Formatter) -> Result;

    /// This function is used to construct a new state. It is called once per state
    fn enter_state(&self, ctx: &Ctx, f: &mut Formatter, state: Uid) -> Result;
    /// This function is used to finish a state. It is called once per state
    fn leave_state(&self, ctx: &Ctx, f: &mut Formatter, state: Uid) -> Result;
    /// This function is used to enter a match case (matching on a token) in a state.
    fn enter_match(&self, ctx: &Ctx, f: &mut Formatter, state: Uid, token: Token) -> Result;
    /// This function is used to finish a match case (matching on a token) in a state.
    fn leave_match(&self, ctx: &Ctx, f: &mut Formatter, state: Uid, token: Token) -> Result;

    /// This function is used to encode a shift action. It is always between an [`enter_match`] a
    /// [`leave_match`]
    fn visit_shift(
        &self,
        ctx: &Ctx,
        f: &mut Formatter,
        state: Uid,
        token: Token,
        next_state: Uid,
    ) -> Result;
    /// This function is used to encode a reduce action. It is always between an [`enter_match`] a
    /// [`leave_match`]
    fn visit_reduce(
        &self,
        ctx: &Ctx,
        f: &mut Formatter,
        state: Uid,
        token: Token,
        rule: Id,
        expansion: &[Token],
    ) -> Result;
    /// This function is used to handle a matching error.
    fn matching_error(
        &self,
        ctx: &Ctx,
        f: &mut Formatter,
        state: Uid,
        expected: HashSet<Token>,
    ) -> Result;
    /// This function is used to set up the goto tables
    fn visit_goto(
        &self,
        ctx: &Ctx,
        f: &mut Formatter,
        symbol: Id,
        gotos: impl Iterator<Item = (Uid, Uid)>,
    ) -> Result;
}

pub struct Render<'a, V> {
    table: &'a Table,
    grammar: &'a Grammar,
    v: V,
}

impl<V> Display for Render<'_, V>
where
    V: Visitor,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let ctx = Ctx {
            grammar: self.grammar,
        };
        let v = &self.v;
        let all_states: Vec<_> = self.table.0.keys().copied().collect();
        v.before_enter(&ctx, f, all_states.as_slice())?;
        let mut gotos = HashMap::new();
        for (node_id, entry) in &self.table.0 {
            for (&sym, &goto) in &entry.gotos {
                gotos
                    .entry(sym)
                    .and_modify(|x: &mut HashMap<usize, usize>| {
                        x.insert(*node_id, goto);
                    })
                    .or_insert_with(|| HashMap::from([(*node_id, goto)]));
            }
        }
        for (symbol, goto) in gotos {
            v.visit_goto(&ctx, f, symbol, goto.into_iter().sorted())?;
        }

        v.begin_parse_loop(&ctx, f)?;

        for (&node_id, entry) in self.table.0.iter().sorted_by_key(|x| x.0) {
            v.enter_state(&ctx, f, node_id)?;

            for (&tok, action) in &entry.actions {
                v.enter_match(&ctx, f, node_id, tok)?;
                match action {
                    Action::Reduce(rule, expansion) => {
                        v.visit_reduce(&ctx, f, node_id, tok, *rule, expansion.as_slice())?
                    }
                    Action::Shift(next_state) => {
                        v.visit_shift(&ctx, f, node_id, tok, *next_state)?
                    }
                }
                v.leave_match(&ctx, f, node_id, tok)?;
            }

            v.matching_error(
                &ctx,
                f,
                node_id,
                entry.actions.iter().map(|(&x, _)| x).collect(),
            )?;

            v.leave_state(&ctx, f, node_id)?;
        }

        v.end_parse_loop(&ctx, f)?;

        v.after_leave(&ctx, f, all_states.as_slice())?;

        Ok(())
    }
}

impl<'a, V> Render<'a, V>
where
    V: Visitor,
{
    pub fn new(v: V, table: &'a Table, grammar: &'a Grammar) -> Self {
        Render { v, table, grammar }
    }
}
