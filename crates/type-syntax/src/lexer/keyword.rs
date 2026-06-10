use crate::token::TypeTokenKind;

/// Fast keyword lookup using two-level dispatch.
/// Returns the TypeTokenKind if the bytes match a keyword (case-insensitive).
#[inline]
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
        25 => lookup_len25(bytes),
        26 => lookup_len26(bytes),
        _ => None,
    }
}

#[inline]
fn eq(a: &[u8], b: &[u8]) -> bool {
    a.eq_ignore_ascii_case(b)
}

#[inline]
fn eq_exact(a: &[u8], b: &[u8]) -> bool {
    a == b
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
            if eq_exact(bytes, b"min") {
                Some(TypeTokenKind::Min)
            } else if eq_exact(bytes, b"max") {
                Some(TypeTokenKind::Max)
            } else {
                None
            }
        }
        b'n' => {
            if eq(bytes, b"not") {
                Some(TypeTokenKind::Not)
            } else if eq(bytes, b"new") {
                Some(TypeTokenKind::New)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[inline]
fn lookup_len4(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'b' if eq(bytes, b"bool") => Some(TypeTokenKind::Bool),
        b'l' if eq(bytes, b"list") => Some(TypeTokenKind::List),
        b'n' if eq(bytes, b"null") => Some(TypeTokenKind::Null),
        b'r' if eq_exact(bytes, b"real") => Some(TypeTokenKind::Real),
        b't' if eq(bytes, b"true") => Some(TypeTokenKind::True),
        b'v' if eq(bytes, b"void") => Some(TypeTokenKind::Void),
        _ => None,
    }
}

#[inline]
fn lookup_len5(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'a' if eq(bytes, b"array") => Some(TypeTokenKind::Array),
        b'e' if eq(bytes, b"empty") => Some(TypeTokenKind::Empty),
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
        b'd' if eq_exact(bytes, b"double") => Some(TypeTokenKind::Double),
        b'k' if eq(bytes, b"key-of") => Some(TypeTokenKind::KeyOf),
        b'o' if eq(bytes, b"object") => Some(TypeTokenKind::Object),
        b's' => {
            if eq_exact(bytes, b"scalar") {
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
        b'b' if eq_exact(bytes, b"boolean") => Some(TypeTokenKind::Boolean),
        b'i' if eq_exact(bytes, b"integer") => Some(TypeTokenKind::Integer),
        b'n' => {
            if eq_exact(bytes, b"nothing") {
                Some(TypeTokenKind::Nothing)
            } else if eq_exact(bytes, b"numeric") {
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
        b'r' if eq_exact(bytes, b"resource") => Some(TypeTokenKind::Resource),
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
        b'e' if eq(bytes, b"empty-scalar") => Some(TypeTokenKind::EmptyScalar),
        b'n' => {
            if eq(bytes, b"never-return") {
                Some(TypeTokenKind::NeverReturn)
            } else if eq(bytes, b"negative-int") {
                Some(TypeTokenKind::NegativeInt)
            } else if eq(bytes, b"non-zero-int") {
                Some(TypeTokenKind::NonZeroInt)
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
        b't' => {
            if eq(bytes, b"truthy-string") {
                Some(TypeTokenKind::TruthyString)
            } else if eq(bytes, b"template-type") {
                Some(TypeTokenKind::TemplateType)
            } else {
                None
            }
        }
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
        b'c' => {
            if eq(bytes, b"closed-resource") {
                Some(TypeTokenKind::ClosedResource)
            } else if eq(bytes, b"callable-string") {
                Some(TypeTokenKind::CallableString)
            } else {
                None
            }
        }
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
fn lookup_len25(bytes: &[u8]) -> Option<TypeTokenKind> {
    match bytes[0] | 0x20 {
        b'l' if eq(bytes, b"lowercase-callable-string") => Some(TypeTokenKind::LowercaseCallableString),
        b'u' if eq(bytes, b"uppercase-callable-string") => Some(TypeTokenKind::UppercaseCallableString),
        _ => None,
    }
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

#[cfg(test)]
mod tests {
    use super::lookup_keyword;
    use crate::token::TypeTokenKind;

    #[test]
    fn class_name_keywords_match_lowercase_only() {
        assert_eq!(lookup_keyword(b"resource"), Some(TypeTokenKind::Resource));
        assert_eq!(lookup_keyword(b"Resource"), None);
        assert_eq!(lookup_keyword(b"RESOURCE"), None);

        assert_eq!(lookup_keyword(b"integer"), Some(TypeTokenKind::Integer));
        assert_eq!(lookup_keyword(b"Integer"), None);

        assert_eq!(lookup_keyword(b"boolean"), Some(TypeTokenKind::Boolean));
        assert_eq!(lookup_keyword(b"Boolean"), None);

        assert_eq!(lookup_keyword(b"double"), Some(TypeTokenKind::Double));
        assert_eq!(lookup_keyword(b"Double"), None);

        assert_eq!(lookup_keyword(b"scalar"), Some(TypeTokenKind::Scalar));
        assert_eq!(lookup_keyword(b"Scalar"), None);

        assert_eq!(lookup_keyword(b"numeric"), Some(TypeTokenKind::Numeric));
        assert_eq!(lookup_keyword(b"Numeric"), None);

        assert_eq!(lookup_keyword(b"nothing"), Some(TypeTokenKind::Nothing));
        assert_eq!(lookup_keyword(b"Nothing"), None);

        assert_eq!(lookup_keyword(b"real"), Some(TypeTokenKind::Real));
        assert_eq!(lookup_keyword(b"Real"), None);

        assert_eq!(lookup_keyword(b"min"), Some(TypeTokenKind::Min));
        assert_eq!(lookup_keyword(b"Min"), None);
        assert_eq!(lookup_keyword(b"max"), Some(TypeTokenKind::Max));
        assert_eq!(lookup_keyword(b"Max"), None);
    }

    #[test]
    fn reserved_keywords_stay_case_insensitive() {
        assert_eq!(lookup_keyword(b"int"), Some(TypeTokenKind::Int));
        assert_eq!(lookup_keyword(b"INT"), Some(TypeTokenKind::Int));
        assert_eq!(lookup_keyword(b"Int"), Some(TypeTokenKind::Int));

        assert_eq!(lookup_keyword(b"string"), Some(TypeTokenKind::String));
        assert_eq!(lookup_keyword(b"STRING"), Some(TypeTokenKind::String));

        assert_eq!(lookup_keyword(b"bool"), Some(TypeTokenKind::Bool));
        assert_eq!(lookup_keyword(b"Bool"), Some(TypeTokenKind::Bool));

        assert_eq!(lookup_keyword(b"callable"), Some(TypeTokenKind::Callable));
        assert_eq!(lookup_keyword(b"Callable"), Some(TypeTokenKind::Callable));
    }
}
