use std::{collections::BTreeSet, fmt::Display};

use itertools::{Either, Itertools};

use crate::{
    generator::State,
    string_pool::{Id, Pool},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    Term(Id),
    NonTerm(Id),
    Empty,
    Eof,
}

pub struct TokenDisplay<'a> {
    token: Token,
    pool: &'a Pool,
}

impl Display for TokenDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.token {
            Token::Term(id) => write!(f, "`{}`", self.pool.get(id))?,
            Token::NonTerm(id) => write!(f, "{}", self.pool.get(id))?,
            Token::Empty => write!(f, "𝛆!")?,
            Token::Eof => write!(f, "＄")?,
        }
        Ok(())
    }
}

impl Token {
    pub fn display<'a>(&self, pool: &'a Pool) -> TokenDisplay<'a> {
        TokenDisplay { token: *self, pool }
    }

    pub fn term(self) -> Option<Id> {
        match self {
            Self::Term(i) => Some(i),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn eof(self) -> Option<()> {
        match self {
            Self::Eof => Some(()),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn non_term(self) -> Option<Id> {
        match self {
            Self::NonTerm(i) => Some(i),
            _ => None,
        }
    }

    pub fn fold<F, G, T>(self, non_term: F, term: G, default: T) -> T
    where
        F: FnOnce(Id) -> T,
        G: FnOnce(Id) -> T,
    {
        match self {
            Token::Term(id) => term(id),
            Token::NonTerm(id) => non_term(id),
            _ => default,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrammarEntry {
    rule_name: Id,
    tokens: Vec<Token>,
    code: String,
}

impl GrammarEntry {
    pub fn rule_name(&self) -> Id {
        self.rule_name
    }

    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn display(&self, f: &mut std::fmt::Formatter<'_>, pool: &Pool) -> std::fmt::Result {
        write!(f, "{} -> ", pool.get(self.rule_name))?;
        if self.tokens.is_empty() {
            write!(f, "𝜖")?;
        }
        for token in &self.tokens {
            write!(f, "{} ", token.display(pool))?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Grammar {
    pool: Pool,
    entries: Vec<GrammarEntry>,
}

impl Display for Grammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for entry in &self.entries {
            entry.display(f, &self.pool)?;
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Grammar {
    pub fn builder() -> GrammarBuilder {
        GrammarBuilder::new()
    }

    pub fn productions(&self, rule: Id) -> impl IntoIterator<Item = &[Token]> {
        self.entries
            .iter()
            .filter_map(move |entry| (entry.rule_name == rule).then_some(entry.tokens.as_slice()))
    }

    pub fn first(&self, rules: &[Token]) -> BTreeSet<Token> {
        match rules {
            [] => BTreeSet::from([Token::Empty]),
            [Token::Term(t), ..] => BTreeSet::from([Token::Term(*t)]),
            [Token::Eof, ..] => BTreeSet::from([Token::Eof]),
            [Token::NonTerm(nt), more @ ..] => {
                let (this, other): (Vec<_>, Vec<_>) = self
                    .productions(*nt)
                    .into_iter()
                    .partition_map(|prod| match prod {
                        [Token::NonTerm(x), rest @ ..] if x == nt => Either::Left(rest),
                        _ => Either::Right(prod),
                    });

                let mut firsts: BTreeSet<_> =
                    other.into_iter().flat_map(|x| self.first(x)).collect();

                if firsts.contains(&Token::Empty) && !this.is_empty() {
                    if !more.is_empty() {
                        firsts.remove(&Token::Empty);
                    }

                    firsts.remove(&Token::Empty);
                    firsts = firsts
                        .union(&this.into_iter().flat_map(|x| self.first(x)).collect())
                        .copied()
                        .collect();
                }

                if firsts.is_empty() || firsts.contains(&Token::Empty) {
                    firsts.remove(&Token::Empty);

                    firsts.union(&self.first(more)).copied().collect()
                } else {
                    firsts
                }
            }
            [_, ..] => unreachable!("rules may not contain empty token"),
        }
    }

    pub fn initial(&self, rule: Id) -> impl IntoIterator<Item = State> + '_ {
        self.productions(rule)
            .into_iter()
            .map(move |x| State::new(rule, x.to_vec()))
    }

    pub fn entries(&self) -> &[GrammarEntry] {
        &self.entries
    }

    pub fn pool_mut(&mut self) -> &mut Pool {
        &mut self.pool
    }

    pub fn pool(&self) -> &Pool {
        &self.pool
    }
}

#[derive(Debug, Clone)]
pub struct GrammarBuilder {
    string_pool: Pool,
    entries: Vec<GrammarEntry>,
}

impl GrammarBuilder {
    pub fn new() -> Self {
        GrammarBuilder {
            string_pool: Pool::new(),
            entries: Vec::new(),
        }
    }

    pub fn production<S>(mut self, rule: S, prod: ProductionBuilt, code: String) -> Self
    where
        S: ToOwned<Owned = String>,
    {
        let rule_name = self.string_pool.add(rule);
        self.entries.push(GrammarEntry {
            rule_name,
            tokens: prod.tokens,
            code,
        });
        self
    }

    pub fn prod_builder(&mut self) -> ProductionBuilder {
        ProductionBuilder {
            pool: &mut self.string_pool,
            tokens: Vec::new(),
        }
    }

    pub fn finish(mut self, entry_point: String) -> Grammar {
        let super_rule = self.string_pool.add("S0".to_owned());
        let entry_point = self.string_pool.add(entry_point);
        self.entries.push(GrammarEntry {
            rule_name: super_rule,
            tokens: Vec::from([Token::NonTerm(entry_point), Token::Eof]),
            code: "".to_owned(),
        });

        Grammar {
            pool: self.string_pool,
            entries: self.entries,
        }
    }
}

pub struct ProductionBuilder<'a> {
    pool: &'a mut Pool,
    tokens: Vec<Token>,
}

pub struct ProductionBuilt {
    tokens: Vec<Token>,
}

impl ProductionBuilder<'_> {
    pub fn term<S>(mut self, term: S) -> Self
    where
        S: ToOwned<Owned = String>,
    {
        let term = self.pool.add(term);
        self.tokens.push(Token::Term(term));
        self
    }

    pub fn non_term<S>(mut self, non_term: S) -> Self
    where
        S: ToOwned<Owned = String>,
    {
        let non_term = self.pool.add(non_term);
        self.tokens.push(Token::NonTerm(non_term));
        self
    }

    pub fn finish(self) -> ProductionBuilt {
        ProductionBuilt {
            tokens: self.tokens,
        }
    }
}

#[macro_export]
macro_rules! prod {
    ($builder:expr, $prod_builder:expr, @ $code:literal $(;)?) => {
        {
            let prod = $prod_builder.finish();
            (prod, $code, $builder)
        }
    };
    ($builder:expr, $prod_builder:expr, @ $code:literal ; $($rest:tt)*) =>
    {
        {
        let prod = $prod_builder.finish();
        let builder = $crate::prod!($builder, None, $($rest)*);
        (prod, $code, builder)
        }
    };
    ($builder:expr, $prod_builder:expr, T $term:literal $($rest:tt)* ) => {
        $crate::prod!($builder, $prod_builder.term($term.to_owned()), $($rest)*)
    };
    ($builder:expr, $prod_builder:expr, N $nterm:literal $($rest:tt)* ) => {
        $crate::prod!($builder, $prod_builder.non_term($nterm.to_owned()), $($rest)*)
    };
    (
        $builder:expr, None, $rule:literal => $($rest:tt)*
    ) => {{
        let (production, code, builder) = prod!($builder, $builder.prod_builder(), $($rest)*);
        builder.production($rule.to_owned(), production, code.to_owned())
    }};
    (
        $builder:expr, None, $rule:ident => $($rest:tt)*
    ) => {{
        let (production, code, builder) = $crate::prod!($builder, $builder.prod_builder(), $($rest)*);
        builder.production(stringify!($rule).to_owned(), production, code.to_owned())
    }};
}

#[macro_export]
macro_rules! grammar {
    ($entry:ident : $($tt:tt)*) => {{
        let mut builder = Grammar::builder();
        let builder = $crate::prod!(builder, None, $($tt)*);
        builder.finish(stringify!($entry).to_owned())
    }};
}
