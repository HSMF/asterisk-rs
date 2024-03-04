use std::collections::HashMap;

use anyhow::Context;
use itertools::Itertools;

use crate::{generator::Uid, grammar::Token, string_pool::Id};

use super::{Ctx, Format, Visitor};

pub struct OcamlVisitor {
    prelude: String,
    non_terminal_types: HashMap<String, String>,
    terminal_types: HashMap<String, String>,
    token_type: String,
    entry_rule: String,
}

impl OcamlVisitor {
    pub fn new(
        prelude: String,
        mut non_terminal_types: HashMap<String, String>,
        terminal_types: HashMap<String, String>,
        entry_rule: String,
    ) -> Self {
        non_terminal_types.insert(
            "S0".to_owned(),
            non_terminal_types
                .get(&entry_rule)
                .expect("entry rule is missing in types")
                .to_owned(),
        );
        OcamlVisitor {
            prelude,
            non_terminal_types,
            terminal_types,
            token_type: "token".to_owned(),
            entry_rule: "S0".to_owned(),
        }
    }
}

impl Visitor for OcamlVisitor {
    fn enter_state(&self, _ctx: &Ctx, f: &mut std::fmt::Formatter, state: Uid) -> std::fmt::Result {
        writeln!(
            f,
            r#"  and node{state} (_stack: stack) (input: token list) ="#
        )?;
        writeln!(f, r#"    match input with"#)?;

        Ok(())
    }

    fn leave_state(&self, _ctx: &Ctx, f: &mut std::fmt::Formatter, state: Uid) -> std::fmt::Result {
        writeln!(f, "     (* end state {state} *)")?;
        Ok(())
    }

    fn visit_shift(
        &self,
        ctx: &Ctx,
        f: &mut std::fmt::Formatter,
        _: Uid,
        token: Token,
        next_state: Uid,
    ) -> std::fmt::Result {
        let (has_data, name) = match token {
            Token::Term(id) => {
                let name = ctx.grammar.pool().get(id);
                (self.terminal_types.contains_key(name), name)
            }
            _ => (false, ""),
        };
        writeln!(
            f,
            "      let stack = (State_{next_state}, {}, {}) :: _stack in",
            if token == Token::Eof {
                "TermEof"
            } else {
                "Term _head"
            },
            if has_data {
                format!("StackValue_Term_{name} _value")
            } else {
                "StackValue_None".to_owned()
            }
        )?;
        writeln!(f, "      node{next_state} stack _input'")?;
        Ok(())
    }

    fn visit_reduce(
        &self,
        ctx: &Ctx,
        f: &mut std::fmt::Formatter,
        _state: Uid,
        _token: Token,
        rule: Id,
        expansion: &[Token],
    ) -> std::fmt::Result {
        let grammar = ctx.grammar;
        let pool = grammar.pool();
        let rule_name = pool.get(rule);

        for (i, token) in expansion.iter().enumerate().rev() {
            writeln!(f, "      let (_, typ, tmp), _stack = pop_stack _stack in")?;
            writeln!(f, "      let v{i} = (match typ, tmp with")?;
            write!(f, "      | ")?;
            let (typ, name) = match token {
                Token::Term(id) => {
                    let name = pool.get(*id);
                    write!(f, "Term {name}")?;
                    if self.terminal_types.contains_key(name) {
                        write!(f, " _")?;
                    }
                    (self.terminal_types.get(name), name)
                }
                Token::NonTerm(id) => {
                    let name = pool.get(*id);
                    write!(f, "NonTerm NonTerm_{name}")?;
                    (self.non_terminal_types.get(name), name)
                }
                Token::Empty => unreachable!(),
                Token::Eof => {
                    write!(f, "TermEof")?;
                    (None, "")
                }
            };

            write!(f, ", ")?;
            match typ {
                None => write!(f, "StackValue_None -> ()")?,
                Some(_) => write!(
                    f,
                    "StackValue_{}_{name} v -> v",
                    token.fold(|_| "Nonterm", |_| "Term", "Term")
                )?,
            }
            writeln!(f)?;
            writeln!(
                f,
                r#"      | _ -> raise_msg ( "expected token {}" )) in"#,
                token.display(pool)
            )?;

            writeln!(f, "      ignore v{i};")?;
        }

        let code = grammar
            .entries()
            .iter()
            .find(|x| x.rule_name() == rule && x.tokens() == expansion)
            .unwrap()
            .code();

        writeln!(f, "      let _value = ({code}) in")?;

        if rule_name == self.entry_rule {
            writeln!(f, "       v0")?;
            return Ok(());
        }

        writeln!(f, "      let (before, _, _) = List.hd _stack in")?;
        writeln!(f, "      let goto, goto_id = goto_{rule_name} before in")?;
        writeln!(f, "      let _stack = (goto_id, NonTerm NonTerm_{rule_name}, StackValue_Nonterm_{rule_name} _value) :: _stack in")?;
        writeln!(f, "      goto _stack input")?;

        Ok(())
    }

    fn matching_error(
        &self,
        _ctx: &Ctx,
        f: &mut std::fmt::Formatter,
        state: Uid,
        _expected: std::collections::HashSet<Token>,
    ) -> std::fmt::Result {
        writeln!(
            f,
            r#"    | _ -> raise (Parse_error (ErrUnexpectedToken ([], "node{state}", input) ))"#
        )?;

        Ok(())
    }

    fn before_enter(
        &self,
        ctx: &Ctx,
        f: &mut std::fmt::Formatter,
        all_states: &[Uid],
    ) -> std::fmt::Result {
        writeln!(
            f,
            r#"
    (* Autogenerated file *)
      {}
      type error_data =
        | ErrMsg of string
        | ErrUnexpectedToken of string list * string * token list
      exception Parse_error of error_data

      type states ="#,
            self.prelude
        )?;

        for state in all_states {
            writeln!(f, "       | State_{state}")?;
        }

        writeln!(f, "  type stack_value =")?;
        let grammar = ctx.grammar;
        let pool = grammar.pool();
        for state in grammar
            .entries()
            .iter()
            .map(|x| x.rule_name())
            .sorted()
            .dedup()
        {
            let name = pool.get(state);
            writeln!(
                f,
                "       | StackValue_Nonterm_{} of ({})",
                name,
                self.non_terminal_types.get(name).expect("undefined type")
            )?;
        }

        for state in grammar
            .entries()
            .iter()
            .flat_map(|x| x.tokens())
            .filter_map(|x| x.term())
            .sorted()
            .dedup()
        {
            let name = pool.get(state);
            if let Some(typ) = self.terminal_types.get(name) {
                writeln!(f, "       | StackValue_Term_{} of ({typ})", name)?;
            }
        }
        writeln!(f, "       | StackValue_None")?;
        writeln!(f, "  type nonterm =")?;
        for state in grammar
            .entries()
            .iter()
            .map(|x| x.rule_name())
            .sorted()
            .dedup()
        {
            let name = pool.get(state);

            writeln!(f, "       | NonTerm_{name}")?;
        }

        writeln!(
            f,
            "  type token_type = Term of {} | NonTerm of nonterm | TermEof",
            self.token_type
        )?;
        writeln!(f, "  type stack = (states * token_type * stack_value) list")?;

        writeln!(
            f,
            r#"
            let parse input =

            let raise_msg m = raise (Parse_error (ErrMsg (m))) in
            let pop msg = function
              | [] -> raise_msg (msg)
              | hd::tl -> hd, tl in
            let pop_stack a = pop "stack" a in

            let rec _hello = ()
            "#
        )?;

        Ok(())
    }

    fn after_leave(&self, ctx: &Ctx, f: &mut std::fmt::Formatter, _: &[Uid]) -> std::fmt::Result {
        writeln!(
            f,
            r#"
        in
        node1 [ State_1, NonTerm NonTerm_{}, StackValue_None ] input
        "#,
            ctx.grammar.pool().get(
                ctx.grammar
                    .entries()
                    .last()
                    .expect("grammar was empty, this should never happen")
                    .rule_name()
            )
        )?;
        Ok(())
    }

    fn visit_goto(
        &self,
        ctx: &Ctx,
        f: &mut std::fmt::Formatter,
        symbol: Id,
        gotos: &mut dyn Iterator<Item = (Uid, Uid)>,
    ) -> std::fmt::Result {
        let grammar = ctx.grammar;
        let name = grammar.pool().get(symbol);
        writeln!(f, "  and goto_{} (state: states) = ", name)?;
        writeln!(f, "    match state with")?;
        for (from, to) in gotos.into_iter() {
            writeln!(f, "    | State_{from} -> node{to}, State_{to}")?;
        }
        writeln!(f, r#"    | _ -> raise_msg ("couldn't match in {}")"#, name)?;
        writeln!(f)?;

        Ok(())
    }

    fn enter_match(
        &self,
        ctx: &Ctx,
        f: &mut std::fmt::Formatter,
        _state: Uid,
        token: Token,
    ) -> std::fmt::Result {
        write!(f, "    | ")?;
        match token {
            Token::Term(term) => {
                let grammar = ctx.grammar;
                let name = grammar.pool().get(term);
                writeln!(
                    f,
                    "({}{}) as _head :: _input' -> begin",
                    name,
                    if self.terminal_types.contains_key(name) {
                        " _value"
                    } else {
                        ""
                    }
                )?;
            }
            Token::Eof => writeln!(f, "[] as _input' -> begin")?,
            _ => unreachable!(),
        }

        Ok(())
    }

    fn leave_match(
        &self,
        _ctx: &Ctx,
        f: &mut std::fmt::Formatter,
        _: Uid,
        _: Token,
    ) -> std::fmt::Result {
        writeln!(f, "    end")?;
        Ok(())
    }

    fn begin_parse_loop(&self, _: &Ctx, _: &mut std::fmt::Formatter) -> std::fmt::Result {
        Ok(())
    }

    fn end_parse_loop(&self, _: &Ctx, _: &mut std::fmt::Formatter) -> std::fmt::Result {
        Ok(())
    }
}

impl Format for OcamlVisitor {
    fn format(&self, path: &str) -> anyhow::Result<()> {
        let mut handle = std::process::Command::new("ocamlformat")
            .arg(path)
            .arg("--inplace")
            .arg("--enable-outside-detected-project")
            .spawn()
            .context("could not spawn ocamlformat")?;
        handle.wait().context("could not wait for ocamlformat")?;
        Ok(())
    }
}
