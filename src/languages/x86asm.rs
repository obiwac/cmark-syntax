use crate::{Highlight, Kind};
use logos::Logos;

#[derive(Logos, PartialEq, Eq, Clone, Copy, Debug)]
pub enum X86Asm {
    #[regex("[a-zA-Z_$][a-zA-Z0-9_]*")]
    Identifier,

    #[regex("\"([^\"\\\\]|\\\\[.\n])*\"")]
    #[regex("'([^']|\\\\')'")]
    #[regex("[0-9][0-9]*")]
    #[regex("0[xX][0-9a-fA-F]+")]
    #[regex("0[bB][01]+")]
    Literal,

    #[regex(r#"\?|:|!|\^|-|\+|\*|&|/|<|>|="#, priority = 3)]
    Glyph,

    #[regex("\\{|\\}|\\[|\\]|\\(|\\)")]
    Bracket,

    #[regex("#(mov|mwait)")]
    Keyword,

    #[regex("#(equ|org|db|dw|dd|dq|dt)")]
    Macro,

    #[regex("#(eax|ebx)")]
    Constant,

    #[regex(";[^\n]*")]
    #[regex("//[^\n]*")]
    #[regex("/\\*([^/]|[^*]/)*\\*/")]
    Comment,

    None,
}

impl Highlight for X86Asm {
    const LANG: &'static str = "x86asm";
    const START: Self = Self::None;

    fn kind(tokens: &[Self; 2]) -> Kind {
        use X86Asm::*;

        match tokens {
            [_, Identifier] => Kind::Identifier,
            [_, Literal] => Kind::Literal,
            [_, Glyph] => Kind::Glyph,
            [_, Keyword] | [_, Constant] | [_, Macro] => Kind::Keyword,
            [_, Comment] => Kind::Comment,
            _ => Kind::None,
        }
    }
}
