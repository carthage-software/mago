#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKeyword {
    As,
    Is,
    Int,
    Min,
    Max,
    Not,
    New,
    Bool,
    List,
    Null,
    Real,
    True,
    Void,
    Empty,
    Self_,
    Array,
    False,
    Float,
    Mixed,
    Never,
    Double,
    KeyOf,
    Static,
    Parent,
    Object,
    Scalar,
    String,
    Boolean,
    Integer,
    Nothing,
    Numeric,
    Callable,
    IntMask,
    Iterable,
    Resource,
    ValueOf,
    ArrayKey,
    NoReturn,
    EmptyScalar,
    EnumString,
    IntMaskOf,
    UnspecifiedLiteralInt,
    ClassString,
    ClassLikeString,
    NeverReturn,
    NegativeInt,
    NonZeroInt,
    PositiveInt,
    PureClosure,
    TraitString,
    UnspecifiedLiteralFloat,
    NeverReturns,
    OpenResource,
    PropertiesOf,
    PureCallable,
    TruthyString,
    TemplateType,
    UnspecifiedLiteralString,
    NonEmptyList,
    NumericString,
    ClosedResource,
    CallableString,
    NonEmptyArray,
    NonEmptyMixed,
    InterfaceString,
    LowercaseString,
    UppercaseString,
    NonEmptyString,
    NonFalsyString,
    NonPositiveInt,
    NonNegativeInt,
    AssociativeArray,
    StringableObject,
    PublicPropertiesOf,
    PrivatePropertiesOf,
    ProtectedPropertiesOf,
    NonEmptyUnspecifiedLiteralString,
    LowercaseCallableString,
    UppercaseCallableString,
    NonEmptyLowercaseString,
    NonEmptyUppercaseString,
}

#[inline]
#[must_use]
pub fn lookup_keyword(bytes: &[u8]) -> Option<TypeKeyword> {
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
fn lookup_len2(bytes: &[u8]) -> Option<TypeKeyword> {
    match bytes[0] | 0x20 {
        b'a' if eq(bytes, b"as") => Some(TypeKeyword::As),
        b'i' if eq(bytes, b"is") => Some(TypeKeyword::Is),
        _ => None,
    }
}

#[inline]
fn lookup_len3(bytes: &[u8]) -> Option<TypeKeyword> {
    match bytes[0] | 0x20 {
        b'i' if eq(bytes, b"int") => Some(TypeKeyword::Int),
        b'm' => {
            if eq_exact(bytes, b"min") {
                Some(TypeKeyword::Min)
            } else if eq_exact(bytes, b"max") {
                Some(TypeKeyword::Max)
            } else {
                None
            }
        }
        b'n' => {
            if eq(bytes, b"not") {
                Some(TypeKeyword::Not)
            } else if eq(bytes, b"new") {
                Some(TypeKeyword::New)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[inline]
fn lookup_len4(bytes: &[u8]) -> Option<TypeKeyword> {
    match bytes[0] | 0x20 {
        b'b' if eq(bytes, b"bool") => Some(TypeKeyword::Bool),
        b'l' if eq(bytes, b"list") => Some(TypeKeyword::List),
        b'n' if eq(bytes, b"null") => Some(TypeKeyword::Null),
        b'r' if eq_exact(bytes, b"real") => Some(TypeKeyword::Real),
        b't' if eq(bytes, b"true") => Some(TypeKeyword::True),
        b'v' if eq(bytes, b"void") => Some(TypeKeyword::Void),
        b's' if eq(bytes, b"self") => Some(TypeKeyword::Self_),
        _ => None,
    }
}

#[inline]
fn lookup_len5(bytes: &[u8]) -> Option<TypeKeyword> {
    match bytes[0] | 0x20 {
        b'e' if eq(bytes, b"empty") => Some(TypeKeyword::Empty),
        b'a' if eq(bytes, b"array") => Some(TypeKeyword::Array),
        b'f' => {
            if eq(bytes, b"false") {
                Some(TypeKeyword::False)
            } else if eq(bytes, b"float") {
                Some(TypeKeyword::Float)
            } else {
                None
            }
        }
        b'm' if eq(bytes, b"mixed") => Some(TypeKeyword::Mixed),
        b'n' if eq(bytes, b"never") => Some(TypeKeyword::Never),
        _ => None,
    }
}

#[inline]
fn lookup_len6(bytes: &[u8]) -> Option<TypeKeyword> {
    match bytes[0] | 0x20 {
        b'd' if eq_exact(bytes, b"double") => Some(TypeKeyword::Double),
        b'k' if eq(bytes, b"key-of") => Some(TypeKeyword::KeyOf),
        b'o' if eq(bytes, b"object") => Some(TypeKeyword::Object),
        b's' => {
            if eq_exact(bytes, b"scalar") {
                Some(TypeKeyword::Scalar)
            } else if eq(bytes, b"string") {
                Some(TypeKeyword::String)
            } else if eq(bytes, b"static") {
                Some(TypeKeyword::Static)
            } else {
                None
            }
        }
        b'p' if eq(bytes, b"parent") => Some(TypeKeyword::Parent),
        _ => None,
    }
}

#[inline]
fn lookup_len7(bytes: &[u8]) -> Option<TypeKeyword> {
    match bytes[0] | 0x20 {
        b'b' if eq_exact(bytes, b"boolean") => Some(TypeKeyword::Boolean),
        b'i' if eq_exact(bytes, b"integer") => Some(TypeKeyword::Integer),
        b'n' => {
            if eq_exact(bytes, b"nothing") {
                Some(TypeKeyword::Nothing)
            } else if eq_exact(bytes, b"numeric") {
                Some(TypeKeyword::Numeric)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[inline]
fn lookup_len8(bytes: &[u8]) -> Option<TypeKeyword> {
    match bytes[0] | 0x20 {
        b'c' if eq(bytes, b"callable") => Some(TypeKeyword::Callable),
        b'i' => {
            if eq(bytes, b"int-mask") {
                Some(TypeKeyword::IntMask)
            } else if eq(bytes, b"iterable") {
                Some(TypeKeyword::Iterable)
            } else {
                None
            }
        }
        b'r' if eq_exact(bytes, b"resource") => Some(TypeKeyword::Resource),
        b'v' if eq(bytes, b"value-of") => Some(TypeKeyword::ValueOf),
        _ => None,
    }
}

#[inline]
fn lookup_len9(bytes: &[u8]) -> Option<TypeKeyword> {
    match bytes[0] | 0x20 {
        b'a' if eq(bytes, b"array-key") => Some(TypeKeyword::ArrayKey),
        b'n' if eq(bytes, b"no-return") => Some(TypeKeyword::NoReturn),
        _ => None,
    }
}

#[inline]
fn lookup_len11(bytes: &[u8]) -> Option<TypeKeyword> {
    match bytes[0] | 0x20 {
        b'e' if eq(bytes, b"enum-string") => Some(TypeKeyword::EnumString),
        b'i' if eq(bytes, b"int-mask-of") => Some(TypeKeyword::IntMaskOf),
        b'l' if eq(bytes, b"literal-int") => Some(TypeKeyword::UnspecifiedLiteralInt),
        _ => None,
    }
}

#[inline]
fn lookup_len12(bytes: &[u8]) -> Option<TypeKeyword> {
    match bytes[0] | 0x20 {
        b'e' if eq(bytes, b"empty-scalar") => Some(TypeKeyword::EmptyScalar),
        b'c' if eq(bytes, b"class-string") => Some(TypeKeyword::ClassString),
        b'n' => {
            if eq(bytes, b"never-return") {
                Some(TypeKeyword::NeverReturn)
            } else if eq(bytes, b"negative-int") {
                Some(TypeKeyword::NegativeInt)
            } else if eq(bytes, b"non-zero-int") {
                Some(TypeKeyword::NonZeroInt)
            } else {
                None
            }
        }
        b'p' => {
            if eq(bytes, b"positive-int") {
                Some(TypeKeyword::PositiveInt)
            } else if eq(bytes, b"pure-closure") {
                Some(TypeKeyword::PureClosure)
            } else {
                None
            }
        }
        b't' if eq(bytes, b"trait-string") => Some(TypeKeyword::TraitString),
        _ => None,
    }
}

#[inline]
fn lookup_len13(bytes: &[u8]) -> Option<TypeKeyword> {
    match bytes[0] | 0x20 {
        b'l' if eq(bytes, b"literal-float") => Some(TypeKeyword::UnspecifiedLiteralFloat),
        b'n' if eq(bytes, b"never-returns") => Some(TypeKeyword::NeverReturns),
        b'o' if eq(bytes, b"open-resource") => Some(TypeKeyword::OpenResource),
        b'p' => {
            if eq(bytes, b"properties-of") {
                Some(TypeKeyword::PropertiesOf)
            } else if eq(bytes, b"pure-callable") {
                Some(TypeKeyword::PureCallable)
            } else {
                None
            }
        }
        b't' => {
            if eq(bytes, b"truthy-string") {
                Some(TypeKeyword::TruthyString)
            } else if eq(bytes, b"template-type") {
                Some(TypeKeyword::TemplateType)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[inline]
fn lookup_len14(bytes: &[u8]) -> Option<TypeKeyword> {
    match bytes[0] | 0x20 {
        b'l' if eq(bytes, b"literal-string") => Some(TypeKeyword::UnspecifiedLiteralString),
        b'n' => {
            if eq(bytes, b"non-empty-list") {
                Some(TypeKeyword::NonEmptyList)
            } else if eq(bytes, b"numeric-string") {
                Some(TypeKeyword::NumericString)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[inline]
fn lookup_len15(bytes: &[u8]) -> Option<TypeKeyword> {
    match bytes[0] | 0x20 {
        b'c' => {
            if eq(bytes, b"closed-resource") {
                Some(TypeKeyword::ClosedResource)
            } else if eq(bytes, b"callable-string") {
                Some(TypeKeyword::CallableString)
            } else {
                None
            }
        }
        b'n' => {
            if eq(bytes, b"non-empty-array") {
                Some(TypeKeyword::NonEmptyArray)
            } else if eq(bytes, b"non-empty-mixed") {
                Some(TypeKeyword::NonEmptyMixed)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[inline]
fn lookup_len16(bytes: &[u8]) -> Option<TypeKeyword> {
    match bytes[0] | 0x20 {
        b'i' if eq(bytes, b"interface-string") => Some(TypeKeyword::InterfaceString),
        b'l' if eq(bytes, b"lowercase-string") => Some(TypeKeyword::LowercaseString),
        b'u' if eq(bytes, b"uppercase-string") => Some(TypeKeyword::UppercaseString),
        b'n' => {
            if eq(bytes, b"non-empty-string") {
                Some(TypeKeyword::NonEmptyString)
            } else if eq(bytes, b"non-falsy-string") {
                Some(TypeKeyword::NonFalsyString)
            } else if eq(bytes, b"non-positive-int") {
                Some(TypeKeyword::NonPositiveInt)
            } else if eq(bytes, b"non-negative-int") {
                Some(TypeKeyword::NonNegativeInt)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[inline]
fn lookup_len17(bytes: &[u8]) -> Option<TypeKeyword> {
    match bytes[0] | 0x20 {
        b'a' if eq(bytes, b"associative-array") => Some(TypeKeyword::AssociativeArray),
        b'c' if eq(bytes, b"class-like-string") => Some(TypeKeyword::ClassLikeString),
        b's' if eq(bytes, b"stringable-object") => Some(TypeKeyword::StringableObject),
        _ => None,
    }
}

#[inline]
fn lookup_len20(bytes: &[u8]) -> Option<TypeKeyword> {
    if eq(bytes, b"public-properties-of") { Some(TypeKeyword::PublicPropertiesOf) } else { None }
}

#[inline]
fn lookup_len21(bytes: &[u8]) -> Option<TypeKeyword> {
    if eq(bytes, b"private-properties-of") { Some(TypeKeyword::PrivatePropertiesOf) } else { None }
}

#[inline]
fn lookup_len23(bytes: &[u8]) -> Option<TypeKeyword> {
    if eq(bytes, b"protected-properties-of") { Some(TypeKeyword::ProtectedPropertiesOf) } else { None }
}

#[inline]
fn lookup_len24(bytes: &[u8]) -> Option<TypeKeyword> {
    if eq(bytes, b"non-empty-literal-string") { Some(TypeKeyword::NonEmptyUnspecifiedLiteralString) } else { None }
}

#[inline]
fn lookup_len25(bytes: &[u8]) -> Option<TypeKeyword> {
    match bytes[0] | 0x20 {
        b'l' if eq(bytes, b"lowercase-callable-string") => Some(TypeKeyword::LowercaseCallableString),
        b'u' if eq(bytes, b"uppercase-callable-string") => Some(TypeKeyword::UppercaseCallableString),
        _ => None,
    }
}

#[inline]
fn lookup_len26(bytes: &[u8]) -> Option<TypeKeyword> {
    if eq(bytes, b"non-empty-lowercase-string") {
        Some(TypeKeyword::NonEmptyLowercaseString)
    } else if eq(bytes, b"non-empty-uppercase-string") {
        Some(TypeKeyword::NonEmptyUppercaseString)
    } else {
        None
    }
}
