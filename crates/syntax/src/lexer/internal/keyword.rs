use crate::token::TokenKind;

/// Fast keyword lookup. Returns the TokenKind if the bytes match a keyword (case-insensitive).
///
/// Uses a two-level dispatch:
///
/// 1. Dispatch on length (eliminates most candidates)
/// 2. Dispatch on first character, then compare remaining bytes
#[inline(always)]
pub fn lookup_keyword(bytes: &[u8]) -> Option<TokenKind> {
    match bytes.len() {
        2 => lookup_len2(bytes),
        3 => lookup_len3(bytes),
        4 => lookup_len4(bytes),
        5 => lookup_len5(bytes),
        6 => lookup_len6(bytes),
        7 => lookup_len7(bytes),
        8 => lookup_len8(bytes),
        9 => lookup_len9(bytes),
        10 => lookup_len10(bytes),
        12 => lookup_len12(bytes),
        13 => lookup_len13(bytes),
        15 => lookup_len15(bytes),
        _ => None,
    }
}

#[inline(always)]
fn eq_ignore_case(a: &[u8], b: &[u8]) -> bool {
    a.eq_ignore_ascii_case(b)
}

#[inline]
fn lookup_len2(bytes: &[u8]) -> Option<TokenKind> {
    match bytes[0] | 0x20 {
        b'a' if eq_ignore_case(bytes, b"as") => Some(TokenKind::As),
        b'd' if eq_ignore_case(bytes, b"do") => Some(TokenKind::Do),
        b'f' if eq_ignore_case(bytes, b"fn") => Some(TokenKind::Fn),
        b'i' if eq_ignore_case(bytes, b"if") => Some(TokenKind::If),
        b'o' if eq_ignore_case(bytes, b"or") => Some(TokenKind::Or),
        _ => None,
    }
}

#[inline]
fn lookup_len3(bytes: &[u8]) -> Option<TokenKind> {
    match bytes[0] | 0x20 {
        b'a' if eq_ignore_case(bytes, b"and") => Some(TokenKind::And),
        b'd' if eq_ignore_case(bytes, b"die") => Some(TokenKind::Die),
        b'f' if eq_ignore_case(bytes, b"for") => Some(TokenKind::For),
        b'n' if eq_ignore_case(bytes, b"new") => Some(TokenKind::New),
        b't' if eq_ignore_case(bytes, b"try") => Some(TokenKind::Try),
        b'u' if eq_ignore_case(bytes, b"use") => Some(TokenKind::Use),
        b'v' if eq_ignore_case(bytes, b"var") => Some(TokenKind::Var),
        b'x' if eq_ignore_case(bytes, b"xor") => Some(TokenKind::Xor),
        _ => None,
    }
}

#[inline]
fn lookup_len4(bytes: &[u8]) -> Option<TokenKind> {
    match bytes[0] | 0x20 {
        b'c' if eq_ignore_case(bytes, b"case") => Some(TokenKind::Case),
        b'e' => match bytes[1] | 0x20 {
            b'c' if eq_ignore_case(bytes, b"echo") => Some(TokenKind::Echo),
            b'l' if eq_ignore_case(bytes, b"else") => Some(TokenKind::Else),
            b'n' if eq_ignore_case(bytes, b"enum") => Some(TokenKind::Enum),
            b'v' if eq_ignore_case(bytes, b"eval") => Some(TokenKind::Eval),
            b'x' if eq_ignore_case(bytes, b"exit") => Some(TokenKind::Exit),
            _ => None,
        },
        b'f' if eq_ignore_case(bytes, b"from") => Some(TokenKind::From),
        b'g' if eq_ignore_case(bytes, b"goto") => Some(TokenKind::Goto),
        b'l' if eq_ignore_case(bytes, b"list") => Some(TokenKind::List),
        b'n' if eq_ignore_case(bytes, b"null") => Some(TokenKind::Null),
        b's' if eq_ignore_case(bytes, b"self") => Some(TokenKind::Self_),
        b't' if eq_ignore_case(bytes, b"true") => Some(TokenKind::True),
        _ => None,
    }
}

#[inline]
fn lookup_len5(bytes: &[u8]) -> Option<TokenKind> {
    match bytes[0] | 0x20 {
        b'a' if eq_ignore_case(bytes, b"array") => Some(TokenKind::Array),
        b'b' if eq_ignore_case(bytes, b"break") => Some(TokenKind::Break),
        b'c' => match bytes[1] | 0x20 {
            b'a' if eq_ignore_case(bytes, b"catch") => Some(TokenKind::Catch),
            b'l' => {
                if eq_ignore_case(bytes, b"class") {
                    Some(TokenKind::Class)
                } else if eq_ignore_case(bytes, b"clone") {
                    Some(TokenKind::Clone)
                } else {
                    None
                }
            }
            b'o' if eq_ignore_case(bytes, b"const") => Some(TokenKind::Const),
            _ => None,
        },
        b'e' => {
            if eq_ignore_case(bytes, b"empty") {
                Some(TokenKind::Empty)
            } else if eq_ignore_case(bytes, b"endif") {
                Some(TokenKind::EndIf)
            } else {
                None
            }
        }
        b'f' => {
            if eq_ignore_case(bytes, b"false") {
                Some(TokenKind::False)
            } else if eq_ignore_case(bytes, b"final") {
                Some(TokenKind::Final)
            } else {
                None
            }
        }
        b'i' if eq_ignore_case(bytes, b"isset") => Some(TokenKind::Isset),
        b'm' if eq_ignore_case(bytes, b"match") => Some(TokenKind::Match),
        b'p' if eq_ignore_case(bytes, b"print") => Some(TokenKind::Print),
        b't' => {
            if eq_ignore_case(bytes, b"throw") {
                Some(TokenKind::Throw)
            } else if eq_ignore_case(bytes, b"trait") {
                Some(TokenKind::Trait)
            } else {
                None
            }
        }
        b'u' if eq_ignore_case(bytes, b"unset") => Some(TokenKind::Unset),
        b'w' if eq_ignore_case(bytes, b"while") => Some(TokenKind::While),
        b'y' if eq_ignore_case(bytes, b"yield") => Some(TokenKind::Yield),
        _ => None,
    }
}

#[inline]
fn lookup_len6(bytes: &[u8]) -> Option<TokenKind> {
    match bytes[0] | 0x20 {
        b'e' => {
            if eq_ignore_case(bytes, b"elseif") {
                Some(TokenKind::ElseIf)
            } else if eq_ignore_case(bytes, b"endfor") {
                Some(TokenKind::EndFor)
            } else {
                None
            }
        }
        b'g' if eq_ignore_case(bytes, b"global") => Some(TokenKind::Global),
        b'p' => {
            if eq_ignore_case(bytes, b"parent") {
                Some(TokenKind::Parent)
            } else if eq_ignore_case(bytes, b"public") {
                Some(TokenKind::Public)
            } else {
                None
            }
        }
        b'r' if eq_ignore_case(bytes, b"return") => Some(TokenKind::Return),
        b's' => {
            if eq_ignore_case(bytes, b"static") {
                Some(TokenKind::Static)
            } else if eq_ignore_case(bytes, b"switch") {
                Some(TokenKind::Switch)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[inline]
fn lookup_len7(bytes: &[u8]) -> Option<TokenKind> {
    match bytes[0] {
        b'_' if eq_ignore_case(bytes, b"__dir__") => Some(TokenKind::DirConstant),
        _ => match bytes[0] | 0x20 {
            b'd' => {
                if eq_ignore_case(bytes, b"declare") {
                    Some(TokenKind::Declare)
                } else if eq_ignore_case(bytes, b"default") {
                    Some(TokenKind::Default)
                } else {
                    None
                }
            }
            b'e' if eq_ignore_case(bytes, b"extends") => Some(TokenKind::Extends),
            b'f' => {
                if eq_ignore_case(bytes, b"finally") {
                    Some(TokenKind::Finally)
                } else if eq_ignore_case(bytes, b"foreach") {
                    Some(TokenKind::Foreach)
                } else {
                    None
                }
            }
            b'i' if eq_ignore_case(bytes, b"include") => Some(TokenKind::Include),
            b'p' if eq_ignore_case(bytes, b"private") => Some(TokenKind::Private),
            b'r' if eq_ignore_case(bytes, b"require") => Some(TokenKind::Require),
            _ => None,
        },
    }
}

#[inline]
fn lookup_len8(bytes: &[u8]) -> Option<TokenKind> {
    match bytes[0] {
        b'_' => {
            if eq_ignore_case(bytes, b"__file__") {
                Some(TokenKind::FileConstant)
            } else if eq_ignore_case(bytes, b"__line__") {
                Some(TokenKind::LineConstant)
            } else {
                None
            }
        }
        _ => match bytes[0] | 0x20 {
            b'a' if eq_ignore_case(bytes, b"abstract") => Some(TokenKind::Abstract),
            b'c' => {
                if eq_ignore_case(bytes, b"callable") {
                    Some(TokenKind::Callable)
                } else if eq_ignore_case(bytes, b"continue") {
                    Some(TokenKind::Continue)
                } else {
                    None
                }
            }
            b'e' if eq_ignore_case(bytes, b"endwhile") => Some(TokenKind::EndWhile),
            b'f' if eq_ignore_case(bytes, b"function") => Some(TokenKind::Function),
            b'r' if eq_ignore_case(bytes, b"readonly") => Some(TokenKind::Readonly),
            _ => None,
        },
    }
}

#[inline]
fn lookup_len9(bytes: &[u8]) -> Option<TokenKind> {
    match bytes[0] {
        b'_' => {
            if eq_ignore_case(bytes, b"__class__") {
                Some(TokenKind::ClassConstant)
            } else if eq_ignore_case(bytes, b"__trait__") {
                Some(TokenKind::TraitConstant)
            } else {
                None
            }
        }
        _ => match bytes[0] | 0x20 {
            b'e' if eq_ignore_case(bytes, b"endswitch") => Some(TokenKind::EndSwitch),
            b'i' => {
                if eq_ignore_case(bytes, b"insteadof") {
                    Some(TokenKind::Insteadof)
                } else if eq_ignore_case(bytes, b"interface") {
                    Some(TokenKind::Interface)
                } else {
                    None
                }
            }
            b'n' if eq_ignore_case(bytes, b"namespace") => Some(TokenKind::Namespace),
            b'p' if eq_ignore_case(bytes, b"protected") => Some(TokenKind::Protected),
            _ => None,
        },
    }
}

#[inline]
fn lookup_len10(bytes: &[u8]) -> Option<TokenKind> {
    match bytes[0] {
        b'_' if eq_ignore_case(bytes, b"__method__") => Some(TokenKind::MethodConstant),
        _ => match bytes[0] | 0x20 {
            b'e' => {
                if eq_ignore_case(bytes, b"enddeclare") {
                    Some(TokenKind::EndDeclare)
                } else if eq_ignore_case(bytes, b"endforeach") {
                    Some(TokenKind::EndForeach)
                } else {
                    None
                }
            }
            b'i' => {
                if eq_ignore_case(bytes, b"implements") {
                    Some(TokenKind::Implements)
                } else if eq_ignore_case(bytes, b"instanceof") {
                    Some(TokenKind::Instanceof)
                } else {
                    None
                }
            }
            _ => None,
        },
    }
}

#[inline]
fn lookup_len12(bytes: &[u8]) -> Option<TokenKind> {
    match bytes[0] {
        b'_' => {
            if eq_ignore_case(bytes, b"__function__") {
                Some(TokenKind::FunctionConstant)
            } else if eq_ignore_case(bytes, b"__property__") {
                Some(TokenKind::PropertyConstant)
            } else {
                None
            }
        }
        _ => match bytes[0] | 0x20 {
            b'i' if eq_ignore_case(bytes, b"include_once") => Some(TokenKind::IncludeOnce),
            b'r' if eq_ignore_case(bytes, b"require_once") => Some(TokenKind::RequireOnce),
            _ => None,
        },
    }
}

#[inline]
fn lookup_len13(bytes: &[u8]) -> Option<TokenKind> {
    if eq_ignore_case(bytes, b"__namespace__") { Some(TokenKind::NamespaceConstant) } else { None }
}

#[inline]
fn lookup_len15(bytes: &[u8]) -> Option<TokenKind> {
    if eq_ignore_case(bytes, b"__halt_compiler") { Some(TokenKind::HaltCompiler) } else { None }
}
