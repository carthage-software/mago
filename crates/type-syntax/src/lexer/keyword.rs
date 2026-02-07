use crate::token::TypeTokenKind;

/// Fast keyword lookup using two-level dispatch.
/// Returns the TypeTokenKind if the bytes match a keyword (case-insensitive).
#[inline(always)]
pub fn lookup_keyword(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes.len() {
        2 => lookup_len2(bytes),
        3 => lookup_len3(bytes),
        4 => lookup_len4(bytes),
        5 => lookup_len5(bytes),
        6 => lookup_len6(bytes),
        7 => lookup_len7(bytes),
        8 => lookup_len8(bytes),
        9 => lookup_len9(bytes),
        11 => lookup_len11(bytes),
        12 => lookup_len12(bytes),
        13 => lookup_len13(bytes),
        14 => lookup_len14(bytes),
        15 => lookup_len15(bytes),
        16 => lookup_len16(bytes),
        17 => lookup_len17(bytes),
        20 => lookup_len20(bytes),
        21 => lookup_len21(bytes),
        23 => lookup_len23(bytes),
        24 => lookup_len24(bytes),
        26 => lookup_len26(bytes),
        _ => None,
    }
}

#[inline(always)]
fn eq(a: &[u8], b: &[u8]) -> bool {
    a.eq_ignore_ascii_case(b)
}

#[inline]
fn lookup_len2(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'a' if eq(bytes, b"as") => Some(TypeTokenKind::As),
        b'i' if eq(bytes, b"is") => Some(TypeTokenKind::Is),
        _ => None,
    }
}

#[inline]
fn lookup_len3(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'i' if eq(bytes, b"int") => Some(TypeTokenKind::Int),
        b'm' => {
            if eq(bytes, b"min") {
                Some(TypeTokenKind::Min)
            } else if eq(bytes, b"max") {
                Some(TypeTokenKind::Max)
            } else {
                None
            }
        }
        b'n' if eq(bytes, b"not") => Some(TypeTokenKind::Not),
        _ => None,
    }
}

#[inline]
fn lookup_len4(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'b' if eq(bytes, b"bool") => Some(TypeTokenKind::Bool),
        b'l' if eq(bytes, b"list") => Some(TypeTokenKind::List),
        b'n' if eq(bytes, b"null") => Some(TypeTokenKind::Null),
        b'r' if eq(bytes, b"real") => Some(TypeTokenKind::Real),
        b't' if eq(bytes, b"true") => Some(TypeTokenKind::True),
        b'v' if eq(bytes, b"void") => Some(TypeTokenKind::Void),
        _ => None,
    }
}

#[inline]
fn lookup_len5(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'a' if eq(bytes, b"array") => Some(TypeTokenKind::Array),
        b'f' => {
            if eq(bytes, b"false") {
                Some(TypeTokenKind::False)
            } else if eq(bytes, b"float") {
                Some(TypeTokenKind::Float)
            } else {
                None
            }
        }
        b'm' if eq(bytes, b"mixed") => Some(TypeTokenKind::Mixed),
        b'n' if eq(bytes, b"never") => Some(TypeTokenKind::Never),
        _ => None,
    }
}

#[inline]
fn lookup_len6(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'd' if eq(bytes, b"double") => Some(TypeTokenKind::Double),
        b'k' if eq(bytes, b"key-of") => Some(TypeTokenKind::KeyOf),
        b'o' if eq(bytes, b"object") => Some(TypeTokenKind::Object),
        b's' => {
            if eq(bytes, b"scalar") {
                Some(TypeTokenKind::Scalar)
            } else if eq(bytes, b"string") {
                Some(TypeTokenKind::String)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[inline]
fn lookup_len7(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'b' if eq(bytes, b"boolean") => Some(TypeTokenKind::Boolean),
        b'i' if eq(bytes, b"integer") => Some(TypeTokenKind::Integer),
        b'n' => {
            if eq(bytes, b"nothing") {
                Some(TypeTokenKind::Nothing)
            } else if eq(bytes, b"numeric") {
                Some(TypeTokenKind::Numeric)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[inline]
fn lookup_len8(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'c' if eq(bytes, b"callable") => Some(TypeTokenKind::Callable),
        b'i' => {
            if eq(bytes, b"int-mask") {
                Some(TypeTokenKind::IntMask)
            } else if eq(bytes, b"iterable") {
                Some(TypeTokenKind::Iterable)
            } else {
                None
            }
        }
        b'r' if eq(bytes, b"resource") => Some(TypeTokenKind::Resource),
        b'v' if eq(bytes, b"value-of") => Some(TypeTokenKind::ValueOf),
        _ => None,
    }
}

#[inline]
fn lookup_len9(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'a' if eq(bytes, b"array-key") => Some(TypeTokenKind::ArrayKey),
        b'n' if eq(bytes, b"no-return") => Some(TypeTokenKind::NoReturn),
        _ => None,
    }
}

#[inline]
fn lookup_len11(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'e' if eq(bytes, b"enum-string") => Some(TypeTokenKind::EnumString),
        b'i' if eq(bytes, b"int-mask-of") => Some(TypeTokenKind::IntMaskOf),
        b'l' if eq(bytes, b"literal-int") => Some(TypeTokenKind::UnspecifiedLiteralInt),
        _ => None,
    }
}

#[inline]
fn lookup_len12(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'c' if eq(bytes, b"class-string") => Some(TypeTokenKind::ClassString),
        b'n' => {
            if eq(bytes, b"never-return") {
                Some(TypeTokenKind::NeverReturn)
            } else if eq(bytes, b"negative-int") {
                Some(TypeTokenKind::NegativeInt)
            } else {
                None
            }
        }
        b'p' => {
            if eq(bytes, b"positive-int") {
                Some(TypeTokenKind::PositiveInt)
            } else if eq(bytes, b"pure-closure") {
                Some(TypeTokenKind::PureClosure)
            } else {
                None
            }
        }
        b't' if eq(bytes, b"trait-string") => Some(TypeTokenKind::TraitString),
        _ => None,
    }
}

#[inline]
fn lookup_len13(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'l' if eq(bytes, b"literal-float") => Some(TypeTokenKind::UnspecifiedLiteralFloat),
        b'n' if eq(bytes, b"never-returns") => Some(TypeTokenKind::NeverReturns),
        b'o' if eq(bytes, b"open-resource") => Some(TypeTokenKind::OpenResource),
        b'p' => {
            if eq(bytes, b"properties-of") {
                Some(TypeTokenKind::PropertiesOf)
            } else if eq(bytes, b"pure-callable") {
                Some(TypeTokenKind::PureCallable)
            } else {
                None
            }
        }
        b't' if eq(bytes, b"truthy-string") => Some(TypeTokenKind::TruthyString),
        _ => None,
    }
}

#[inline]
fn lookup_len14(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'l' if eq(bytes, b"literal-string") => Some(TypeTokenKind::UnspecifiedLiteralString),
        b'n' => {
            if eq(bytes, b"non-empty-list") {
                Some(TypeTokenKind::NonEmptyList)
            } else if eq(bytes, b"numeric-string") {
                Some(TypeTokenKind::NumericString)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[inline]
fn lookup_len15(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'c' if eq(bytes, b"closed-resource") => Some(TypeTokenKind::ClosedResource),
        b'n' => {
            if eq(bytes, b"non-empty-array") {
                Some(TypeTokenKind::NonEmptyArray)
            } else if eq(bytes, b"non-empty-mixed") {
                Some(TypeTokenKind::NonEmptyMixed)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[inline]
fn lookup_len16(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'i' if eq(bytes, b"interface-string") => Some(TypeTokenKind::InterfaceString),
        b'l' if eq(bytes, b"lowercase-string") => Some(TypeTokenKind::LowercaseString),
        b'u' if eq(bytes, b"uppercase-string") => Some(TypeTokenKind::UppercaseString),
        b'n' => {
            if eq(bytes, b"non-empty-string") {
                Some(TypeTokenKind::NonEmptyString)
            } else if eq(bytes, b"non-falsy-string") {
                Some(TypeTokenKind::NonFalsyString)
            } else if eq(bytes, b"non-positive-int") {
                Some(TypeTokenKind::NonPositiveInt)
            } else if eq(bytes, b"non-negative-int") {
                Some(TypeTokenKind::NonNegativeInt)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[inline]
fn lookup_len17(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'a' if eq(bytes, b"associative-array") => Some(TypeTokenKind::AssociativeArray),
        b's' if eq(bytes, b"stringable-object") => Some(TypeTokenKind::StringableObject),
        _ => None,
    }
}

#[inline]
fn lookup_len20(bytes: &[u8]) -> Option<TypeTokenKind> {
    if eq(bytes, b"public-properties-of") { Some(TypeTokenKind::PublicPropertiesOf) } else { None }
}

#[inline]
fn lookup_len21(bytes: &[u8]) -> Option<TypeTokenKind> {
    if eq(bytes, b"private-properties-of") { Some(TypeTokenKind::PrivatePropertiesOf) } else { None }
}

#[inline]
fn lookup_len23(bytes: &[u8]) -> Option<TypeTokenKind> {
    if eq(bytes, b"protected-properties-of") { Some(TypeTokenKind::ProtectedPropertiesOf) } else { None }
}

#[inline]
fn lookup_len24(bytes: &[u8]) -> Option<TypeTokenKind> {
    if eq(bytes, b"non-empty-literal-string") { Some(TypeTokenKind::NonEmptyUnspecifiedLiteralString) } else { None }
}

#[inline]
fn lookup_len26(bytes: &[u8]) -> Option<TypeTokenKind> {
    if eq(bytes, b"non-empty-lowercase-string") {
        Some(TypeTokenKind::NonEmptyLowercaseString)
    } else if eq(bytes, b"non-empty-uppercase-string") {
        Some(TypeTokenKind::NonEmptyUppercaseString)
    } else {
        None
    }
}
