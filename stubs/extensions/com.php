<?php

/**
 * @var int
 */
const CLSCTX_INPROC_SERVER = UNKNOWN;

/**
 * @var int
 */
const CLSCTX_INPROC_HANDLER = UNKNOWN;

/**
 * @var int
 */
const CLSCTX_LOCAL_SERVER = UNKNOWN;

/**
 * @var int
 */
const CLSCTX_REMOTE_SERVER = UNKNOWN;

/**
 * @var int
 */
const CLSCTX_SERVER = UNKNOWN;

/**
 * @var int
 */
const CLSCTX_ALL = UNKNOWN;

/**
 * @var int
 */
const VT_NULL = UNKNOWN;

/**
 * @var int
 */
const VT_EMPTY = UNKNOWN;

/**
 * @var int
 */
const VT_UI1 = UNKNOWN;

/**
 * @var int
 */
const VT_I1 = UNKNOWN;

/**
 * @var int
 */
const VT_UI2 = UNKNOWN;

/**
 * @var int
 */
const VT_I2 = UNKNOWN;

/**
 * @var int
 */
const VT_UI4 = UNKNOWN;

/**
 * @var int
 */
const VT_I4 = UNKNOWN;

/**
 * @var int
 */
const VT_R4 = UNKNOWN;

/**
 * @var int
 */
const VT_R8 = UNKNOWN;

/**
 * @var int
 */
const VT_BOOL = UNKNOWN;

/**
 * @var int
 */
const VT_ERROR = UNKNOWN;

/**
 * @var int
 */
const VT_CY = UNKNOWN;

/**
 * @var int
 */
const VT_DATE = UNKNOWN;

/**
 * @var int
 */
const VT_BSTR = UNKNOWN;

/**
 * @var int
 */
const VT_DECIMAL = UNKNOWN;

/**
 * @var int
 */
const VT_UNKNOWN = UNKNOWN;

/**
 * @var int
 */
const VT_DISPATCH = UNKNOWN;

/**
 * @var int
 */
const VT_VARIANT = UNKNOWN;

/**
 * @var int
 */
const VT_INT = UNKNOWN;

/**
 * @var int
 */
const VT_UINT = UNKNOWN;

/**
 * @var int
 */
const VT_ARRAY = UNKNOWN;

/**
 * @var int
 */
const VT_BYREF = UNKNOWN;

/**
 * @var int
 */
const CP_ACP = UNKNOWN;

/**
 * @var int
 */
const CP_MACCP = UNKNOWN;

/**
 * @var int
 */
const CP_OEMCP = UNKNOWN;

/**
 * @var int
 */
const CP_UTF7 = UNKNOWN;

/**
 * @var int
 */
const CP_UTF8 = UNKNOWN;

/**
 * @var int
 */
const CP_SYMBOL = UNKNOWN;

/**
 * @var int
 */
const CP_THREAD_ACP = UNKNOWN;

/**
 * @var int
 */
const VARCMP_LT = UNKNOWN;

/**
 * @var int
 */
const VARCMP_EQ = UNKNOWN;

/**
 * @var int
 */
const VARCMP_GT = UNKNOWN;

/**
 * @var int
 */
const VARCMP_NULL = UNKNOWN;

/**
 * @var int
 */
const LOCALE_SYSTEM_DEFAULT = UNKNOWN;

/**
 * @var int
 */
const LOCALE_NEUTRAL = UNKNOWN;

/**
 * @var int
 */
const NORM_IGNORECASE = UNKNOWN;

/**
 * @var int
 */
const NORM_IGNORENONSPACE = UNKNOWN;

/**
 * @var int
 */
const NORM_IGNORESYMBOLS = UNKNOWN;

/**
 * @var int
 */
const NORM_IGNOREWIDTH = UNKNOWN;

/**
 * @var int
 */
const NORM_IGNOREKANATYPE = UNKNOWN;

// ifdef NORM_IGNOREKASHIDA
/**
 * @var int
 */
const NORM_IGNOREKASHIDA = UNKNOWN;

// endif

/**
 * @var int
 */
const DISP_E_DIVBYZERO = UNKNOWN;

/**
 * @var int
 */
const DISP_E_OVERFLOW = UNKNOWN;

/**
 * @var int
 */
const DISP_E_BADINDEX = UNKNOWN;

/**
 * @var int
 */
const DISP_E_PARAMNOTFOUND = UNKNOWN;

/**
 * @var int
 */
const MK_E_UNAVAILABLE = UNKNOWN;

/**
 * @var int
 */
const VT_UI8 = UNKNOWN;

/**
 * @var int
 */
const VT_I8 = UNKNOWN;

function variant_set(variant $variant, mixed $value): void
{
}

function variant_add(mixed $left, mixed $right): variant
{
}

function variant_cat(mixed $left, mixed $right): variant
{
}

function variant_sub(mixed $left, mixed $right): variant
{
}

function variant_mul(mixed $left, mixed $right): variant
{
}

function variant_and(mixed $left, mixed $right): variant
{
}

function variant_div(mixed $left, mixed $right): variant
{
}

function variant_eqv(mixed $left, mixed $right): variant
{
}

function variant_idiv(mixed $left, mixed $right): variant
{
}

function variant_imp(mixed $left, mixed $right): variant
{
}

function variant_mod(mixed $left, mixed $right): variant
{
}

function variant_or(mixed $left, mixed $right): variant
{
}

function variant_pow(mixed $left, mixed $right): variant
{
}

function variant_xor(mixed $left, mixed $right): variant
{
}

function variant_abs(mixed $value): variant
{
}

function variant_fix(mixed $value): variant
{
}

function variant_int(mixed $value): variant
{
}

function variant_neg(mixed $value): variant
{
}

function variant_not(mixed $value): variant
{
}

function variant_round(mixed $value, int $decimals): null|variant
{
}

function variant_cmp(mixed $left, mixed $right, int $locale_id = LOCALE_SYSTEM_DEFAULT, int $flags = 0): int
{
}

function variant_date_to_timestamp(variant $variant): null|int
{
}

function variant_date_from_timestamp(int $timestamp): variant
{
}

function variant_get_type(variant $variant): int
{
}

function variant_set_type(variant $variant, int $type): void
{
}

function variant_cast(variant $variant, int $type): variant
{
}

function com_get_active_object(string $prog_id, null|int $codepage = null): variant
{
}

function com_create_guid(): string|false
{
}

function com_event_sink(variant $variant, object $sink_object, array|string|null $sink_interface = null): bool
{
}

function com_print_typeinfo(
    variant|string $variant,
    null|string $dispatch_interface = null,
    bool $display_sink = false,
): bool {
}

function com_message_pump(int $timeout_milliseconds = 0): bool
{
}

function com_load_typelib(string $typelib, bool $case_insensitive = true): bool
{
}

/**
 * @not-serializable
 */
class variant
{
    public function __construct(mixed $value = null, int $type = VT_EMPTY, int $codepage = CP_ACP) {}
}

class com extends variant
{
    public function __construct(
        string $module_name,
        array|string|null $server_name = null,
        int $codepage = CP_ACP,
        string $typelib = '',
    ) {}
}

class dotnet extends variant
{
    public function __construct(string $assembly_name, string $datatype_name, int $codepage = CP_ACP) {}
}

final class com_safearray_proxy
{
}

final class com_exception extends Exception
{
}
