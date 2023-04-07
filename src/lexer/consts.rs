use std::fmt::Display;

pub static TYPESCRIPT_KEYWORDS: &[&str] = &[
    "abstract",
    "any",
    "as",
    "asserts",
    "bigint",
    "boolean",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "declare",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "from",
    "function",
    "get",
    "global",
    "if",
    "implements",
    "import",
    "in",
    "infer",
    "instanceof",
    "interface",
    "is",
    "keyof",
    "let",
    "module",
    "namespace",
    "never",
    "new",
    "null",
    "number",
    "object",
    "of",
    "package",
    "private",
    "protected",
    "public",
    "readonly",
    "require",
    "return",
    "set",
    "static",
    "string",
    "super",
    "switch",
    "symbol",
    "this",
    "throw",
    "true",
    "try",
    "type",
    "typeof",
    "unique",
    "unknown",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

#[derive(Debug, Clone)]
pub struct LiteralToken<'a> {
    pub token_kind: TokenKind,
    pub text: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    TokenKeyword,
    TokenComment,
    TokenSymbol,
    TokenStar,
    TokenInvalid,
    TokenOpenCurly,
    TokenCloseCurly,
    TokenOpenParen,
    TokenCloseParen,
    TokenComma,
}

pub static LITERAL_TOKENS: &[LiteralToken] = &[
    LiteralToken {
        text: "{",
        token_kind: TokenKind::TokenOpenCurly,
    },
    LiteralToken {
        text: "}",
        token_kind: TokenKind::TokenCloseCurly,
    },
    LiteralToken {
        text: "(",
        token_kind: TokenKind::TokenOpenParen,
    },
    LiteralToken {
        text: ")",
        token_kind: TokenKind::TokenCloseParen,
    },
    LiteralToken {
        text: ",",
        token_kind: TokenKind::TokenComma,
    },
];