//! `T![...]` - ergonomic matching against [`TwigTokenKind`](crate::token::TwigTokenKind).
//!
//! # Examples
//!
//! ```no_run
//! use mago_twig_syntax::T;
//! use mago_twig_syntax::token::TwigTokenKind;
//!
//! fn is_infix(kind: TwigTokenKind) -> bool {
//!     matches!(kind, T!["+" | "-" | "*" | "/"])
//! }
//! ```

/// Map a punctuation / operator lexeme or keyword to its
/// [`TwigTokenKind`](crate::token::TwigTokenKind) variant. The macro
/// accepts a single lexeme (`T!["=="]`) or an or-pattern spanning several
/// (`T!["+" | "-"]`), suitable for use inside `match` arms and
/// `matches!`.
#[macro_export]
macro_rules! T {
    [ $first:tt $( | $rest:tt )+ ] => {
        $crate::T![$first] $( | $crate::T![$rest] )+
    };

    ["("]  => { $crate::token::TwigTokenKind::LeftParen };
    [")"]  => { $crate::token::TwigTokenKind::RightParen };
    ["["]  => { $crate::token::TwigTokenKind::LeftBracket };
    ["]"]  => { $crate::token::TwigTokenKind::RightBracket };
    ["{"]  => { $crate::token::TwigTokenKind::LeftBrace };
    ["}"]  => { $crate::token::TwigTokenKind::RightBrace };
    [","]  => { $crate::token::TwigTokenKind::Comma };
    [":"]  => { $crate::token::TwigTokenKind::Colon };
    ["."]  => { $crate::token::TwigTokenKind::Dot };
    ["?"]  => { $crate::token::TwigTokenKind::Question };
    ["|"]  => { $crate::token::TwigTokenKind::Pipe };
    ["="]  => { $crate::token::TwigTokenKind::Equal };
    ["=>"] => { $crate::token::TwigTokenKind::FatArrow };

    ["+"]  => { $crate::token::TwigTokenKind::Plus };
    ["-"]  => { $crate::token::TwigTokenKind::Minus };
    ["*"]  => { $crate::token::TwigTokenKind::Asterisk };
    ["/"]  => { $crate::token::TwigTokenKind::Slash };
    ["%"]  => { $crate::token::TwigTokenKind::Percent };
    ["**"] => { $crate::token::TwigTokenKind::AsteriskAsterisk };
    ["//"] => { $crate::token::TwigTokenKind::SlashSlash };

    ["=="]  => { $crate::token::TwigTokenKind::EqualEqual };
    ["!="]  => { $crate::token::TwigTokenKind::BangEqual };
    ["==="] => { $crate::token::TwigTokenKind::EqualEqualEqual };
    ["!=="] => { $crate::token::TwigTokenKind::BangEqualEqual };
    ["<"]   => { $crate::token::TwigTokenKind::LessThan };
    [">"]   => { $crate::token::TwigTokenKind::GreaterThan };
    ["<="]  => { $crate::token::TwigTokenKind::LessThanEqual };
    [">="]  => { $crate::token::TwigTokenKind::GreaterThanEqual };
    ["<=>"] => { $crate::token::TwigTokenKind::Spaceship };

    ["~"]   => { $crate::token::TwigTokenKind::Tilde };
    [".."]  => { $crate::token::TwigTokenKind::DotDot };
    ["..."] => { $crate::token::TwigTokenKind::DotDotDot };
    ["??"]  => { $crate::token::TwigTokenKind::QuestionQuestion };
    ["?:"]  => { $crate::token::TwigTokenKind::QuestionColon };
    ["?."]  => { $crate::token::TwigTokenKind::QuestionDot };

    // Block / variable delimiters: trim-neutral forms. Use the named variant
    // constants directly for dash/tilde variants.
    ["{%"] => { $crate::token::TwigTokenKind::OpenBlock };
    ["%}"] => { $crate::token::TwigTokenKind::CloseBlock };
    ["{{"] => { $crate::token::TwigTokenKind::OpenVariable };
    ["}}"] => { $crate::token::TwigTokenKind::CloseVariable };

    ["and"]         => { $crate::token::TwigTokenKind::And };
    ["or"]          => { $crate::token::TwigTokenKind::Or };
    ["xor"]         => { $crate::token::TwigTokenKind::Xor };
    ["b-and"]       => { $crate::token::TwigTokenKind::BAnd };
    ["b-or"]        => { $crate::token::TwigTokenKind::BOr };
    ["b-xor"]       => { $crate::token::TwigTokenKind::BXor };
    ["in"]          => { $crate::token::TwigTokenKind::In };
    ["not in"]      => { $crate::token::TwigTokenKind::NotIn };
    ["is"]          => { $crate::token::TwigTokenKind::Is };
    ["not"]         => { $crate::token::TwigTokenKind::Not };
    ["matches"]     => { $crate::token::TwigTokenKind::Matches };
    ["starts with"] => { $crate::token::TwigTokenKind::StartsWith };
    ["ends with"]   => { $crate::token::TwigTokenKind::EndsWith };
    ["has some"]    => { $crate::token::TwigTokenKind::HasSome };
    ["has every"]   => { $crate::token::TwigTokenKind::HasEvery };
    ["same as"]     => { $crate::token::TwigTokenKind::SameAs };
    ["divisible by"]=> { $crate::token::TwigTokenKind::DivisibleBy };
}
