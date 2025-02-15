pub const ANONYMOUS_CLASS_NAME: &str = "class@anonymous";

pub const CONSTRUCTOR_MAGIC_METHOD: &str = "__construct";
pub const DESTRUCTOR_MAGIC_METHOD: &str = "__destruct";
pub const CLONE_MAGIC_METHOD: &str = "__clone";
pub const CALL_MAGIC_METHOD: &str = "__call";
pub const CALL_STATIC_MAGIC_METHOD: &str = "__callStatic";
pub const GET_MAGIC_METHOD: &str = "__get";
pub const SET_MAGIC_METHOD: &str = "__set";
pub const ISSET_MAGIC_METHOD: &str = "__isset";
pub const UNSET_MAGIC_METHOD: &str = "__unset";
pub const SLEEP_MAGIC_METHOD: &str = "__sleep";
pub const WAKEUP_MAGIC_METHOD: &str = "__wakeup";
pub const SERIALIZE_MAGIC_METHOD: &str = "__serialize";
pub const UNSERIALIZE_MAGIC_METHOD: &str = "__unserialize";
pub const TO_STRING_MAGIC_METHOD: &str = "__toString";
pub const INVOKE_MAGIC_METHOD: &str = "__invoke";
pub const SET_STATE_MAGIC_METHOD: &str = "__set_state";
pub const DEBUG_INFO_MAGIC_METHOD: &str = "__debugInfo";

pub const MAGIC_METHODS: &[&str] = &[
    CONSTRUCTOR_MAGIC_METHOD,
    DESTRUCTOR_MAGIC_METHOD,
    CLONE_MAGIC_METHOD,
    CALL_MAGIC_METHOD,
    CALL_STATIC_MAGIC_METHOD,
    GET_MAGIC_METHOD,
    SET_MAGIC_METHOD,
    ISSET_MAGIC_METHOD,
    UNSET_MAGIC_METHOD,
    SLEEP_MAGIC_METHOD,
    WAKEUP_MAGIC_METHOD,
    SERIALIZE_MAGIC_METHOD,
    UNSERIALIZE_MAGIC_METHOD,
    TO_STRING_MAGIC_METHOD,
    INVOKE_MAGIC_METHOD,
    SET_STATE_MAGIC_METHOD,
    DEBUG_INFO_MAGIC_METHOD,
];

pub const STRICT_TYPES_DECLARE_DIRECTIVE: &str = "strict_types";

pub const ENCODING_DECLARE_DIRECTIVE: &str = "encoding";

pub const TICKS_DECLARE_DIRECTIVE: &str = "ticks";

pub const DECLARE_DIRECTIVES: [&str; 3] =
    [STRICT_TYPES_DECLARE_DIRECTIVE, ENCODING_DECLARE_DIRECTIVE, TICKS_DECLARE_DIRECTIVE];

// a list of soft reserved keywords in PHP, minus the ones that symbols are allowed to use as names
pub const SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED: [&str; 7] =
    ["parent", "self", "true", "false", "list", "null", "readonly"];

// a list of reserved keywords in PHP
pub const RESERVED_KEYWORDS: [&str; 77] = [
    "static",
    "abstract",
    "final",
    "for",
    "private",
    "protected",
    "public",
    "include",
    "include_once",
    "eval",
    "require",
    "require_once",
    "or",
    "xor",
    "and",
    "instanceof",
    "new",
    "clone",
    "exit",
    "die",
    "if",
    "elseif",
    "else",
    "endif",
    "echo",
    "do",
    "while",
    "endwhile",
    "endfor",
    "foreach",
    "endforeach",
    "declare",
    "enddeclare",
    "as",
    "try",
    "catch",
    "finally",
    "throw",
    "use",
    "insteadof",
    "global",
    "var",
    "unset",
    "isset",
    "empty",
    "continue",
    "goto",
    "function",
    "const",
    "return",
    "print",
    "yield",
    "list",
    "switch",
    "endswitch",
    "case",
    "default",
    "break",
    "array",
    "callable",
    "extends",
    "implements",
    "namespace",
    "trait",
    "interface",
    "class",
    "__CLASS__",
    "__TRAIT__",
    "__FUNCTION__",
    "__METHOD__",
    "__LINE__",
    "__FILE__",
    "__DIR__",
    "__NAMESPACE__",
    "__halt_compiler",
    "fn",
    "match",
];
