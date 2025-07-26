<?php

class PhpToken implements Stringable
{
    public int $id;

    public string $text;

    public int $line;

    public int $pos;

    final public function __construct(int $id, string $text, int $line = -1, int $pos = -1) {}

    /**
     * @return string|null
     */
    public function getTokenName(): null|string
    {
    }

    /**
     * @return list<PhpToken>
     */
    public static function tokenize(string $code, int $flags = 0): array
    {
    }

    /**
     * @param int|string|array $kind
     */
    public function is($kind): bool
    {
    }

    public function isIgnorable(): bool
    {
    }

    public function __toString(): string
    {
    }
}

/**
 * @return list<array{int, non-empty-string, int}|non-empty-string>
 *
 * @pure
 */
function token_get_all(string $code, int $flags = 0): array
{
}

/**
 * @return non-empty-string
 */
function token_name(int $id): string
{
}

const TOKEN_PARSE = 1;

const T_REQUIRE_ONCE = 263;

const T_REQUIRE = 262;

const T_EVAL = 323;

const T_INCLUDE_ONCE = 261;

const T_INCLUDE = 260;

const T_LOGICAL_OR = 264;

const T_LOGICAL_XOR = 265;

const T_LOGICAL_AND = 266;

const T_PRINT = 267;

const T_YIELD = 268;

const T_DOUBLE_ARROW = 269;

const T_YIELD_FROM = 270;

const T_POW_EQUAL = 282;

const T_SR_EQUAL = 281;

const T_SL_EQUAL = 280;

const T_XOR_EQUAL = 279;

const T_OR_EQUAL = 278;

const T_AND_EQUAL = 277;

const T_MOD_EQUAL = 276;

const T_CONCAT_EQUAL = 275;

const T_DIV_EQUAL = 274;

const T_MUL_EQUAL = 273;

const T_MINUS_EQUAL = 272;

const T_PLUS_EQUAL = 271;

const T_COALESCE_EQUAL = 283;

const T_COALESCE = 284;

const T_BOOLEAN_OR = 285;

const T_BOOLEAN_AND = 286;

const T_SPACESHIP = 293;

const T_IS_NOT_IDENTICAL = 292;

const T_IS_IDENTICAL = 291;

const T_IS_NOT_EQUAL = 290;

const T_IS_EQUAL = 289;

const T_IS_GREATER_OR_EQUAL = 295;

const T_IS_SMALLER_OR_EQUAL = 294;

const T_SR = 297;

const T_SL = 296;

const T_INSTANCEOF = 298;

const T_UNSET_CAST = 305;

const T_BOOL_CAST = 304;

const T_OBJECT_CAST = 303;

const T_ARRAY_CAST = 302;

const T_STRING_CAST = 301;

const T_DOUBLE_CAST = 300;

const T_INT_CAST = 299;

const T_DEC = 389;

const T_INC = 388;

const T_POW = 306;

const T_CLONE = 307;

const T_NEW = 324;

const T_ELSEIF = 309;

const T_ELSE = 310;

const T_ENDIF = 327;

const T_PUBLIC = 362;

const T_PROTECTED = 361;

const T_PRIVATE = 360;

const T_FINAL = 359;

const T_ABSTRACT = 358;

const T_STATIC = 357;

const T_LNUMBER = 311;

const T_DNUMBER = 312;

const T_STRING = 313;

const T_VARIABLE = 317;

const T_INLINE_HTML = 318;

const T_ENCAPSED_AND_WHITESPACE = 319;

const T_CONSTANT_ENCAPSED_STRING = 320;

const T_STRING_VARNAME = 321;

const T_NUM_STRING = 322;

const T_EXIT = 325;

const T_IF = 326;

const T_ECHO = 328;

const T_DO = 329;

const T_WHILE = 330;

const T_ENDWHILE = 331;

const T_FOR = 332;

const T_ENDFOR = 333;

const T_FOREACH = 334;

const T_ENDFOREACH = 335;

const T_DECLARE = 336;

const T_ENDDECLARE = 337;

const T_AS = 338;

const T_SWITCH = 339;

const T_ENDSWITCH = 340;

const T_CASE = 341;

const T_DEFAULT = 342;

const T_MATCH = 343;

const T_BREAK = 344;

const T_CONTINUE = 345;

const T_GOTO = 346;

const T_FUNCTION = 347;

const T_CONST = 349;

const T_RETURN = 350;

const T_TRY = 351;

const T_CATCH = 352;

const T_FINALLY = 353;

const T_THROW = 258;

const T_USE = 354;

const T_INSTEADOF = 355;

const T_GLOBAL = 356;

const T_VAR = 364;

const T_UNSET = 365;

const T_ISSET = 366;

const T_EMPTY = 367;

const T_HALT_COMPILER = 368;

const T_CLASS = 369;

const T_TRAIT = 370;

const T_INTERFACE = 371;

const T_ENUM = 372;

const T_EXTENDS = 373;

const T_IMPLEMENTS = 374;

const T_OBJECT_OPERATOR = 390;

const T_LIST = 376;

const T_ARRAY = 377;

const T_CALLABLE = 378;

const T_LINE = 379;

const T_FILE = 380;

const T_DIR = 381;

const T_CLASS_C = 382;

const T_TRAIT_C = 383;

const T_METHOD_C = 384;

const T_FUNC_C = 385;

const T_NS_C = 386;

const T_PROPERTY_C = 350;

const T_ATTRIBUTE = 387;

const T_COMMENT = 392;

const T_DOC_COMMENT = 393;

const T_OPEN_TAG = 394;

const T_OPEN_TAG_WITH_ECHO = 395;

const T_CLOSE_TAG = 396;

const T_WHITESPACE = 397;

const T_START_HEREDOC = 398;

const T_END_HEREDOC = 399;

const T_DOLLAR_OPEN_CURLY_BRACES = 400;

const T_CURLY_OPEN = 401;

const T_PAAMAYIM_NEKUDOTAYIM = 402;

const T_NAMESPACE = 375;

const T_NS_SEPARATOR = 403;

const T_ELLIPSIS = 404;

const T_DOUBLE_COLON = 402;

const T_FN = 348;

const T_BAD_CHARACTER = 405;

const T_NAME_FULLY_QUALIFIED = 314;

const T_NAME_RELATIVE = 315;

const T_NAME_QUALIFIED = 316;

const T_NULLSAFE_OBJECT_OPERATOR = 391;

const T_AMPERSAND_FOLLOWED_BY_VAR_OR_VARARG = 288;

const T_AMPERSAND_NOT_FOLLOWED_BY_VAR_OR_VARARG = 287;

const T_READONLY = 363;

const T_PRIVATE_SET = 327;

const T_PROTECTED_SET = 328;

const T_PUBLIC_SET = 329;
