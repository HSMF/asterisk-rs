from enum import Enum
import parser

TokenKind = Enum(
    "TokenKind",
    [
        "OPEN_PAREN" "CLOSE_PAREN",
        "INT",
        "PLUS",
        "MINUS",
        "MUL",
        "DIV",
    ],
)


class Token:
    def __init__(self, kind: TokenKind, data=None) -> None:
        self.token_kind = kind
        if data is not None:
            self.data = data

    def kind(self):
        return self.token_kind

    def get_data(self):
        if self.kind() == TokenKind.INT:
            return self.data

        return None


def lex(s: str):
    out = []
    for ch in s:
        if ch == "+":
            out.append(Token(TokenKind.PLUS))
        elif ch == "-":
            out.append(Token(TokenKind.MINUS))
        elif ch == "*":
            out.append(Token(TokenKind.MUL))
        elif ch == "/":
            out.append(Token(TokenKind.DIV))
        elif ch == "(":
            out.append(Token(TokenKind.OPEN_PAREN))
        elif ch == ")":
            out.append(Token(TokenKind.CLOSE_PAREN))
        elif "0" <= ch <= "9":
            out.append(Token(TokenKind.INT, ord(ch) - ord("0")))


def case(input: str, expected: int):
    toks = lex(input)
    result = parser.parse(toks)
    assert result == expected, f"expected {result} = {expected}, {input:=}"


if __name__ == "__main__":
    case("11+2", 13)
    case("1+1", 2)
    case("2*(7+1)", 16)
    case("2*7+1", 15)
    case("21/7+5", 8)
    case("(((((((((((((5)))))))))+1))))-10", -4)
    case("1+2+3+4+5+6+7+8+9+10", 55)
