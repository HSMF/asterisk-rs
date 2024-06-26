import parser
from tokens import Token, TokenKind


def lex(s: str):
    out = []
    for ch in s:
        if ch == "+":
            out.append(Token(TokenKind.Plus))
        elif ch == "-":
            out.append(Token(TokenKind.Minus))
        elif ch == "*":
            out.append(Token(TokenKind.Mul))
        elif ch == "/":
            out.append(Token(TokenKind.Div))
        elif ch == "(":
            out.append(Token(TokenKind.OpenParen))
        elif ch == ")":
            out.append(Token(TokenKind.CloseParen))
        elif "0" <= ch <= "9":
            out.append(Token(TokenKind.Int, ord(ch) - ord("0")))
    return out


def case(input: str, expected: int):
    toks = lex(input)
    result = parser.parse(toks)
    assert result == expected, f"expected {result} = {expected}, {input:=}"
    print(f"SUCCESS: {input} = {result}")


def case_err(input: str):
    toks = lex(input)
    try:
        result = parser.parse(toks)
    except Exception:
        return
    raise Exception(f"expected to fail but did not (returned {result} instead)")


if __name__ == "__main__":
    case("11+2", 13)
    case("1+1", 2)
    case("2*(7+1)", 16)
    case("2*7+1", 15)
    case("21/7+5", 8)
    case("(((((((((((((5)))))))))+1))))-10", -4)
    case("1+2+3+4+5+6+7+8+9+10", 55)

    case_err("1232((9))")
    case_err("1+2+")
    case_err("7*1+8+(")
    case_err(")()(*)(1)(2)(*)(*)")
    case_err("1++2")
    case_err("1*+2")
    case_err("-1")
    case_err("11**(1)")
    case_err("11**(1)")
    case_err("(1+(1+(1+(1+(1+(1+(1+(1+(1+(1+(1+(1+))))))))))))1")
    case_err("(1+)1")
