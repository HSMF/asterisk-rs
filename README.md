# asterisk-rs

neat parser generator, now faster than [asterisk](https://github.com/HSMF/asterisk)

## Installation

Clone this repo and run `cargo install --path .`

## Grammar Specification File

for an example, see [examples/mini_lang](./examples/mini_lang/src/grammar.ast)

```asterisk
# tell asterisk to emit rust
target = rust

# main_rule is the entry point
entry = main_rule

# this code is inserted verbatim before anything else
prelude = {
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum Token {
    OpenParen,
    CloseParen,
    Plus,
    Mul,
    Const(i32),
  }
}

# our terminals are called `Token`
type_token = Token

# the `Const` token carries data, of type i32
token_Const = i32

# this is the entry point, an expression produced by this is of type i32
main_rule: { i32 }
  | main_rule Plus other_rule { v0 + v2 }
  # after matching a sub expression with `main_rule`, a literal `Plus`
  # and a sub expression with `other_rule`, `v0` is populated with the
  # value of `main_rule,` `v1` with `Plus` (which contains no data), and
  # `v2` with `other_rule`. The expression in the braces should be a
  # valid rust expression
  | other_rule { v0 }

other_rule: { i32 }
  | other_rule Mul atom { v0 * v2 }
  | atom { v0 }

atom: { i32 }
  | OpenParen main_rule CloseParen { v1 }
  | Const { v0 } # since the const token carries data, we can use that
                 # data now
```

A "literal" is a block of code, that is interpreted literally by asterisk. It is surrounded by `{` and `}`. It may include the characters `{` and `}` but they must be balanced: `{ {} }` is hence a valid
literal while `{ } }` is not.

Comments start with a `#` and go to the end of the line.

Due to internal reasons, `S0` is currently a disallowed identifier.
