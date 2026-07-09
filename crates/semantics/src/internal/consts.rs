pub const ANONYMOUS_CLASS_NAME: &[u8] = b"class@anonymous";

pub const CONSTRUCTOR_MAGIC_METHOD: &[u8] = b"__construct";
pub const DESTRUCTOR_MAGIC_METHOD: &[u8] = b"__destruct";
pub const CLONE_MAGIC_METHOD: &[u8] = b"__clone";
pub const CALL_MAGIC_METHOD: &[u8] = b"__call";
pub const CALL_STATIC_MAGIC_METHOD: &[u8] = b"__callStatic";
pub const GET_MAGIC_METHOD: &[u8] = b"__get";
pub const SET_MAGIC_METHOD: &[u8] = b"__set";
pub const ISSET_MAGIC_METHOD: &[u8] = b"__isset";
pub const UNSET_MAGIC_METHOD: &[u8] = b"__unset";
pub const SLEEP_MAGIC_METHOD: &[u8] = b"__sleep";
pub const WAKEUP_MAGIC_METHOD: &[u8] = b"__wakeup";
pub const SERIALIZE_MAGIC_METHOD: &[u8] = b"__serialize";
pub const UNSERIALIZE_MAGIC_METHOD: &[u8] = b"__unserialize";
pub const TO_STRING_MAGIC_METHOD: &[u8] = b"__toString";
pub const INVOKE_MAGIC_METHOD: &[u8] = b"__invoke";
pub const SET_STATE_MAGIC_METHOD: &[u8] = b"__set_state";
pub const DEBUG_INFO_MAGIC_METHOD: &[u8] = b"__debugInfo";

// Enums forbid all magic methods except `__call`, `__callStatic` and `__invoke`.
pub const ENUM_FORBIDDEN_MAGIC_METHODS: &[&[u8]] = &[
    CONSTRUCTOR_MAGIC_METHOD,
    DESTRUCTOR_MAGIC_METHOD,
    CLONE_MAGIC_METHOD,
    GET_MAGIC_METHOD,
    SET_MAGIC_METHOD,
    ISSET_MAGIC_METHOD,
    UNSET_MAGIC_METHOD,
    SLEEP_MAGIC_METHOD,
    WAKEUP_MAGIC_METHOD,
    SERIALIZE_MAGIC_METHOD,
    UNSERIALIZE_MAGIC_METHOD,
    TO_STRING_MAGIC_METHOD,
    SET_STATE_MAGIC_METHOD,
    DEBUG_INFO_MAGIC_METHOD,
];

pub const STRICT_TYPES_DECLARE_DIRECTIVE: &[u8] = b"strict_types";

pub const ENCODING_DECLARE_DIRECTIVE: &[u8] = b"encoding";

pub const TICKS_DECLARE_DIRECTIVE: &[u8] = b"ticks";

pub const DECLARE_DIRECTIVES: [&[u8]; 3] =
    [STRICT_TYPES_DECLARE_DIRECTIVE, ENCODING_DECLARE_DIRECTIVE, TICKS_DECLARE_DIRECTIVE];

// a list of soft reserved keywords in PHP, minus the ones that symbols are allowed to use as names
pub const SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED: [&[u8]; 7] =
    [b"parent", b"self", b"true", b"false", b"list", b"null", b"readonly"];

// a list of reserved keywords in PHP
pub const RESERVED_KEYWORDS: [&[u8]; 78] = [
    b"static",
    b"abstract",
    b"final",
    b"for",
    b"private",
    b"protected",
    b"public",
    b"include",
    b"include_once",
    b"eval",
    b"require",
    b"require_once",
    b"or",
    b"xor",
    b"and",
    b"instanceof",
    b"new",
    b"clone",
    b"exit",
    b"die",
    b"if",
    b"elseif",
    b"else",
    b"endif",
    b"echo",
    b"do",
    b"while",
    b"endwhile",
    b"endfor",
    b"foreach",
    b"endforeach",
    b"declare",
    b"enddeclare",
    b"as",
    b"try",
    b"catch",
    b"finally",
    b"throw",
    b"use",
    b"insteadof",
    b"global",
    b"var",
    b"unset",
    b"isset",
    b"empty",
    b"continue",
    b"goto",
    b"function",
    b"const",
    b"return",
    b"print",
    b"yield",
    b"list",
    b"switch",
    b"endswitch",
    b"case",
    b"default",
    b"break",
    b"array",
    b"callable",
    b"extends",
    b"implements",
    b"namespace",
    b"trait",
    b"interface",
    b"class",
    b"__CLASS__",
    b"__TRAIT__",
    b"__FUNCTION__",
    b"__METHOD__",
    b"__LINE__",
    b"__FILE__",
    b"__DIR__",
    b"__NAMESPACE__",
    b"__PROPERTY__",
    b"__halt_compiler",
    b"fn",
    b"match",
];
