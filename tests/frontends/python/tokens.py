from enum import Enum


TokenKind = Enum(
    "TokenKind",
    [
        "OpenParen",
        "CloseParen",
        "Int",
        "Plus",
        "Minus",
        "Mul",
        "Div",
    ],
)


class Token:
    def __init__(self, kind: TokenKind, data=None) -> None:
        self.token_kind = kind
        if data is not None:
            self.data = data

    def get_kind(self):
        return self.token_kind

    def get_data(self):
        if self.get_kind() == TokenKind.Int:
            return self.data

        return None
