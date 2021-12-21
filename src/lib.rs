// This file is part of cmark-syntax. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with cmark-syntax.  If not, see <http://www.gnu.org/licenses/>
#![doc = include_str!("../README.md")]
use logos::Logos;
use pulldown_cmark::{CodeBlockKind, Event, Tag};

/// Definition of syntaxes of various languages.
pub mod languages;

/// A type of token that can be highlighted.
pub trait Highlight: Sized {
    /// Name of the language of this highlighter.
    const LANG: &'static str;

    /// Determine the kind of a token from the current and the previous token.
    fn kind(tokens: &[Self; 2]) -> Kind;
}

/// Possible kind of a token in the highlighted syntax.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Not contained in any tags.
    None,
    /// Rendered among `u` tags.
    Glyph,
    /// Rendered among `span` tags.
    Literal,
    /// Rendered among `var` tags.
    Identifier,
    /// Rendered among `em` tags.
    SpecialIdentifier,
    /// Rendered among `strong` tags.
    StrongIdentifier,
    /// Rendered among `b` tags.
    Keyword,
    /// Rendered among `i` tags.
    Comment,
}

static HIGHLIGHT_TAG: [Option<&'static str>; 8] = {
    let mut tags = [None; 8];

    tags[Kind::Glyph as usize] = Some("u");
    tags[Kind::Literal as usize] = Some("span");
    tags[Kind::Identifier as usize] = Some("var");
    tags[Kind::SpecialIdentifier as usize] = Some("em");
    tags[Kind::StrongIdentifier as usize] = Some("strong");
    tags[Kind::Keyword as usize] = Some("b");
    tags[Kind::Comment as usize] = Some("i");

    tags
};

/// A preprocessor that highlights syntax in `pulldown_cmark` events.
#[derive(Debug, Default)]
pub struct SyntaxPreprocessor<'a, I: Iterator<Item = Event<'a>>> {
    parent: I,
}

impl<'a, I: Iterator<Item = Event<'a>>> SyntaxPreprocessor<'a, I> {
    /// Create a new syntax preprocessor from `parent`.
    pub fn new(parent: I) -> Self {
        Self { parent }
    }
}

impl<'a, I: Iterator<Item = Event<'a>>> Iterator for SyntaxPreprocessor<'a, I> {
    type Item = Event<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let lang = match self.parent.next()? {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => lang,
            other => return Some(other),
        };

        let next_events = (self.parent.next(), self.parent.next());
        let code = if let (Some(Event::Text(ref code)), Some(Event::End(Tag::CodeBlock(_)))) =
            next_events
        {
            code
        } else {
            return Some(Event::Text(format!("Unexpected events {:#?}", next_events).into()));
        };

        #[cfg(feature = "latex2mathml")]
        if lang.as_ref() == "math" {
            return Some(Event::Html(
                latex2mathml::latex_to_mathml(code, latex2mathml::DisplayStyle::Inline)
                    .unwrap_or_else(|e| e.to_string())
                    .into(),
            ));
        } else if lang.as_ref() == "mathblock" {
            return Some(Event::Html(
                latex2mathml::latex_to_mathml(code, latex2mathml::DisplayStyle::Block)
                    .unwrap_or_else(|e| e.to_string())
                    .into(),
            ));
        }

        let mut html = String::with_capacity(code.len() + code.len() / 4 + 60);
        html.push_str("<pre><code class=\"language-");
        html.push_str(lang.as_ref());
        html.push_str("\">");

        match lang.as_ref() {
            "rust" | "rs" => highlight::<languages::Rust>(code, &mut html),
            "js" | "javascript" => highlight::<languages::JavaScript>(code, &mut html),
            "toml" => highlight::<languages::Toml>(code, &mut html),
            "sh" | "shell" | "bash" => highlight::<languages::Sh>(code, &mut html),
            _ => write_escaped(&mut html, code),
        }

        html.push_str("</code></pre>");

        Some(Event::Html(html.into()))
    }
}

/// Write with escaping special HTML characters
#[inline]
fn write_escaped(s: &mut String, part: &str) {
    let mut start = 0;

    for (idx, byte) in part.bytes().enumerate() {
        let replace = match byte {
            b'<' => "&lt;",
            b'>' => "&gt;",
            b'&' => "&amp;",
            b'"' => "&quot;",
            _ => continue,
        };
        s.push_str(&part[start..idx]);
        s.push_str(replace);

        start = idx + 1;
    }

    s.push_str(&part[start..]);
}

/// Highlight the code in `source`, placing the output into `buf`.
#[inline]
pub fn highlight<'a, Token>(source: &'a str, buf: &mut String)
where
    Token: Highlight + Logos<'a, Source = str> + Eq + Copy,
    Token::Extras: Default,
{
    let mut lex = Token::lexer(source);
    let mut open = Kind::None;
    let mut last = 0usize;
    let mut tokens = [Token::ERROR; 2];

    while let Some(token) = lex.next() {
        if tokens[1] != Token::ERROR {
            tokens[0] = tokens[1];
        }
        tokens[1] = token;

        let kind = Token::kind(&tokens);

        if open != kind {
            // Close previous tag
            if let Some(tag) = HIGHLIGHT_TAG[open as usize] {
                buf.push_str("</");
                buf.push_str(tag);
                buf.push('>');
            }

            // Include trivia
            write_escaped(buf, &source[last..lex.span().start]);

            // Open new tag
            if let Some(tag) = HIGHLIGHT_TAG[kind as usize] {
                buf.push('<');
                buf.push_str(tag);
                buf.push('>');
            }

            open = kind;

            write_escaped(buf, lex.slice());
        } else {
            // Include trivia
            write_escaped(buf, &source[last..lex.span().end]);
        }

        last = lex.span().end;
    }

    // Close tail tag
    if let Some(tag) = HIGHLIGHT_TAG[open as usize] {
        buf.push_str("</");
        buf.push_str(tag);
        buf.push('>');
    }
}
