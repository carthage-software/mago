<?php

const __COMPILER_HALT_OFFSET__ = 0;

const CONNECTION_ABORTED = 1;

const CONNECTION_NORMAL = 0;

const CONNECTION_TIMEOUT = 2;

const INI_USER = 1;

const INI_PERDIR = 2;

const INI_SYSTEM = 4;

const INI_ALL = 7;

const INI_SCANNER_NORMAL = 0;

const INI_SCANNER_TYPED = 2;

const INI_SCANNER_RAW = 1;

const PHP_URL_SCHEME = 0;

const PHP_URL_HOST = 1;

const PHP_URL_PORT = 2;

const PHP_URL_USER = 3;

const PHP_URL_PASS = 4;

const PHP_URL_PATH = 5;

const PHP_URL_QUERY = 6;

const PHP_URL_FRAGMENT = 7;

const M_E = 2.718281828459;

const M_LOG2E = 1.442695040889;

const M_LOG10E = 0.43429448190325;

const M_LN2 = 0.69314718055995;

const M_LN10 = 2.302585092994;

const M_PI = 3.1415926535898;

const M_PI_2 = 1.5707963267949;

const M_PI_4 = 0.78539816339745;

const M_1_PI = 0.31830988618379;

const M_2_PI = 0.63661977236758;

const M_SQRTPI = 1.7724538509055;

const M_2_SQRTPI = 1.1283791670955;

const M_LNPI = 1.1447298858494;

const M_EULER = 0.57721566490153;

const M_SQRT2 = 1.4142135623731;

const M_SQRT1_2 = 0.70710678118655;

const M_SQRT3 = 1.7320508075689;

const INF = (float) INF;

const NAN = (float) NAN;

const PHP_ROUND_HALF_UP = 1;

const PHP_ROUND_HALF_DOWN = 2;

const PHP_ROUND_HALF_EVEN = 3;

const PHP_ROUND_HALF_ODD = 4;

const INFO_GENERAL = 1;

const INFO_CREDITS = 2;

const INFO_CONFIGURATION = 4;

const INFO_MODULES = 8;

const INFO_ENVIRONMENT = 16;

const INFO_VARIABLES = 32;

const INFO_LICENSE = 64;

const INFO_ALL = 4294967295;

const CREDITS_GROUP = 1;

const CREDITS_GENERAL = 2;

const CREDITS_SAPI = 4;

const CREDITS_MODULES = 8;

const CREDITS_DOCS = 16;

const CREDITS_FULLPAGE = 32;

const CREDITS_QA = 64;

const CREDITS_ALL = 4294967295;

const HTML_SPECIALCHARS = 0;

const HTML_ENTITIES = 1;

const ENT_COMPAT = 2;

const ENT_QUOTES = 3;

const ENT_NOQUOTES = 0;

const ENT_IGNORE = 4;

const STR_PAD_LEFT = 0;

const STR_PAD_RIGHT = 1;

const STR_PAD_BOTH = 2;

const PATHINFO_DIRNAME = 1;

const PATHINFO_BASENAME = 2;

const PATHINFO_EXTENSION = 4;

const PATHINFO_FILENAME = 8;

const PATHINFO_ALL = 15;

const CHAR_MAX = 127;

const LC_CTYPE = 0;

const LC_NUMERIC = 1;

const LC_TIME = 2;

const LC_COLLATE = 3;

const LC_MONETARY = 4;

const LC_ALL = 6;

const LC_MESSAGES = 5;

const SEEK_SET = 0;

const SEEK_CUR = 1;

const SEEK_END = 2;

const LOCK_SH = 1;

const LOCK_EX = 2;

const LOCK_UN = 3;

const LOCK_NB = 4;

const STREAM_NOTIFY_CONNECT = 2;

const STREAM_NOTIFY_AUTH_REQUIRED = 3;

const STREAM_NOTIFY_AUTH_RESULT = 10;

const STREAM_NOTIFY_MIME_TYPE_IS = 4;

const STREAM_NOTIFY_FILE_SIZE_IS = 5;

const STREAM_NOTIFY_REDIRECTED = 6;

const STREAM_NOTIFY_PROGRESS = 7;

const STREAM_NOTIFY_FAILURE = 9;

const STREAM_NOTIFY_COMPLETED = 8;

const STREAM_NOTIFY_RESOLVE = 1;

const STREAM_NOTIFY_SEVERITY_INFO = 0;

const STREAM_NOTIFY_SEVERITY_WARN = 1;

const STREAM_NOTIFY_SEVERITY_ERR = 2;

const STREAM_FILTER_READ = 1;

const STREAM_FILTER_WRITE = 2;

const STREAM_FILTER_ALL = 3;

const STREAM_CLIENT_PERSISTENT = 1;

const STREAM_CLIENT_ASYNC_CONNECT = 2;

const STREAM_CLIENT_CONNECT = 4;

const STREAM_SHUT_RD = 0;

const STREAM_SHUT_WR = 1;

const STREAM_SHUT_RDWR = 2;

const STREAM_PF_INET = 2;

const STREAM_PF_INET6 = 10;

const STREAM_PF_UNIX = 1;

const STREAM_IPPROTO_IP = 0;

const STREAM_IPPROTO_TCP = 6;

const STREAM_IPPROTO_UDP = 17;

const STREAM_IPPROTO_ICMP = 1;

const STREAM_IPPROTO_RAW = 255;

const STREAM_SOCK_STREAM = 1;

const STREAM_SOCK_DGRAM = 2;

const STREAM_SOCK_RAW = 3;

const STREAM_SOCK_SEQPACKET = 5;

const STREAM_SOCK_RDM = 4;

const STREAM_PEEK = 2;

const STREAM_OOB = 1;

const STREAM_SERVER_BIND = 4;

const STREAM_SERVER_LISTEN = 8;

const FILE_USE_INCLUDE_PATH = 1;

const FILE_IGNORE_NEW_LINES = 2;

const FILE_SKIP_EMPTY_LINES = 4;

const FILE_APPEND = 8;

const FILE_NO_DEFAULT_CONTEXT = 16;

/**
 * @deprecated
 */
const FILE_TEXT = 0;

const FILE_BINARY = 0;

const FNM_NOESCAPE = 2;

const FNM_PATHNAME = 1;

const FNM_PERIOD = 4;

const FNM_CASEFOLD = 16;

const PSFS_PASS_ON = 2;

const PSFS_FEED_ME = 1;

const PSFS_ERR_FATAL = 0;

const PSFS_FLAG_NORMAL = 0;

const PSFS_FLAG_FLUSH_INC = 1;

const PSFS_FLAG_FLUSH_CLOSE = 2;

const ABDAY_1 = 131072;

const ABDAY_2 = 131073;

const ABDAY_3 = 131074;

const ABDAY_4 = 131075;

const ABDAY_5 = 131076;

const ABDAY_6 = 131077;

const ABDAY_7 = 131078;

const DAY_1 = 131079;

const DAY_2 = 131080;

const DAY_3 = 131081;

const DAY_4 = 131082;

const DAY_5 = 131083;

const DAY_6 = 131084;

const DAY_7 = 131085;

const ABMON_1 = 131086;

const ABMON_2 = 131087;

const ABMON_3 = 131088;

const ABMON_4 = 131089;

const ABMON_5 = 131090;

const ABMON_6 = 131091;

const ABMON_7 = 131092;

const ABMON_8 = 131093;

const ABMON_9 = 131094;

const ABMON_10 = 131095;

const ABMON_11 = 131096;

const ABMON_12 = 131097;

const MON_1 = 131098;

const MON_2 = 131099;

const MON_3 = 131100;

const MON_4 = 131101;

const MON_5 = 131102;

const MON_6 = 131103;

const MON_7 = 131104;

const MON_8 = 131105;

const MON_9 = 131106;

const MON_10 = 131107;

const MON_11 = 131108;

const MON_12 = 131109;

const AM_STR = 131110;

const PM_STR = 131111;

const D_T_FMT = 131112;

const D_FMT = 131113;

const T_FMT = 131114;

const T_FMT_AMPM = 131115;

const ERA = 131116;

const ERA_D_T_FMT = 131120;

const ERA_D_FMT = 131118;

const ERA_T_FMT = 131121;

const ALT_DIGITS = 131119;

const CRNCYSTR = 262159;

const RADIXCHAR = 65536;

const THOUSEP = 65537;

const YESEXPR = 327680;

const NOEXPR = 327681;

const YESSTR = 327682;

const NOSTR = 327683;

const CODESET = 14;

const CRYPT_SALT_LENGTH = 123;

const CRYPT_STD_DES = 1;

const CRYPT_EXT_DES = 1;

const CRYPT_MD5 = 1;

const CRYPT_BLOWFISH = 1;

const CRYPT_SHA256 = 1;

const CRYPT_SHA512 = 1;

const DIRECTORY_SEPARATOR = DIRECTORY_SEPARATOR;

const PATH_SEPARATOR = PATH_SEPARATOR;

const GLOB_BRACE = 1024;

const GLOB_MARK = 2;

const GLOB_NOSORT = 4;

const GLOB_NOCHECK = 16;

const GLOB_NOESCAPE = 64;

const GLOB_ERR = 1;

const GLOB_ONLYDIR = 1073741824;

const GLOB_AVAILABLE_FLAGS = 1073741911;

const EXTR_OVERWRITE = 0;

const EXTR_SKIP = 1;

const EXTR_PREFIX_SAME = 2;

const EXTR_PREFIX_ALL = 3;

const EXTR_PREFIX_INVALID = 4;

const EXTR_PREFIX_IF_EXISTS = 5;

const EXTR_IF_EXISTS = 6;

const EXTR_REFS = 256;

const SORT_ASC = 4;

const SORT_DESC = 3;

const SORT_REGULAR = 0;

const SORT_NUMERIC = 1;

const SORT_STRING = 2;

const SORT_LOCALE_STRING = 5;

const CASE_LOWER = 0;

const CASE_UPPER = 1;

const COUNT_NORMAL = 0;

const COUNT_RECURSIVE = 1;

const ASSERT_ACTIVE = 1;

const ASSERT_CALLBACK = 2;

const ASSERT_BAIL = 3;

const ASSERT_WARNING = 4;

const ASSERT_EXCEPTION = 5;

const STREAM_USE_PATH = 1;

const STREAM_IGNORE_URL = 2;

const STREAM_ENFORCE_SAFE_MODE = 4;

const STREAM_REPORT_ERRORS = 8;

const STREAM_MUST_SEEK = 16;

const STREAM_URL_STAT_LINK = 1;

const STREAM_URL_STAT_QUIET = 2;

const STREAM_MKDIR_RECURSIVE = 1;

const STREAM_IS_URL = 1;

const STREAM_OPTION_BLOCKING = 1;

const STREAM_OPTION_READ_TIMEOUT = 4;

const STREAM_OPTION_READ_BUFFER = 2;

const STREAM_OPTION_WRITE_BUFFER = 3;

const STREAM_BUFFER_NONE = 0;

const STREAM_BUFFER_LINE = 1;

const STREAM_BUFFER_FULL = 2;

const STREAM_CAST_AS_STREAM = 0;

const STREAM_CAST_FOR_SELECT = 3;

const IMAGETYPE_GIF = 1;

const IMAGETYPE_JPEG = 2;

const IMAGETYPE_PNG = 3;

const IMAGETYPE_SWF = 4;

const IMAGETYPE_PSD = 5;

const IMAGETYPE_BMP = 6;

const IMAGETYPE_TIFF_II = 7;

const IMAGETYPE_TIFF_MM = 8;

const IMAGETYPE_JPC = 9;

const IMAGETYPE_JP2 = 10;

const IMAGETYPE_JPX = 11;

const IMAGETYPE_JB2 = 12;

const IMAGETYPE_SWC = 13;

const IMAGETYPE_IFF = 14;

const IMAGETYPE_WBMP = 15;

const IMAGETYPE_JPEG2000 = 9;

const IMAGETYPE_XBM = 16;

const IMAGETYPE_ICO = 17;

const IMAGETYPE_WEBP = 18;

const IMAGETYPE_UNKNOWN = 0;

const IMAGETYPE_COUNT = 20;

const IMAGETYPE_AVIF = 19;

const DNS_A = 1;

const DNS_CAA = 8192;

const DNS_NS = 2;

const DNS_CNAME = 16;

const DNS_SOA = 32;

const DNS_PTR = 2048;

const DNS_HINFO = 4096;

const DNS_MX = 16384;

const DNS_TXT = 32768;

const DNS_SRV = 33554432;

const DNS_NAPTR = 67108864;

const DNS_AAAA = 134217728;

const DNS_A6 = 16777216;

const DNS_ANY = 268435456;

const DNS_ALL = 251721779;

const PHP_QUERY_RFC1738 = 1;

const PHP_QUERY_RFC3986 = 2;

const SID = 'name=ID';

const PHP_SESSION_DISABLED = 0;

const PHP_SESSION_NONE = 1;

const PHP_SESSION_ACTIVE = 2;

const ENT_SUBSTITUTE = 8;

const ENT_DISALLOWED = 128;

const ENT_HTML401 = 0;

const ENT_XML1 = 16;

const ENT_XHTML = 32;

const ENT_HTML5 = 48;

const SCANDIR_SORT_ASCENDING = 0;

const SCANDIR_SORT_DESCENDING = 1;

const SCANDIR_SORT_NONE = 2;

const SORT_NATURAL = 6;

const SORT_FLAG_CASE = 8;

const STREAM_META_TOUCH = 1;

const STREAM_META_OWNER = 3;

const STREAM_META_OWNER_NAME = 2;

const STREAM_META_GROUP = 5;

const STREAM_META_GROUP_NAME = 4;

const STREAM_META_ACCESS = 6;

const STREAM_CRYPTO_METHOD_SSLv2_CLIENT = 3;

const STREAM_CRYPTO_METHOD_SSLv3_CLIENT = 5;

const STREAM_CRYPTO_METHOD_SSLv23_CLIENT = 57;

const STREAM_CRYPTO_METHOD_TLS_CLIENT = 121;

const STREAM_CRYPTO_METHOD_SSLv2_SERVER = 2;

const STREAM_CRYPTO_METHOD_SSLv3_SERVER = 4;

const STREAM_CRYPTO_METHOD_SSLv23_SERVER = 120;

const STREAM_CRYPTO_METHOD_TLS_SERVER = 120;

const STREAM_CRYPTO_METHOD_ANY_CLIENT = 127;

const STREAM_CRYPTO_METHOD_ANY_SERVER = 126;

const STREAM_CRYPTO_METHOD_TLSv1_0_CLIENT = 9;

const STREAM_CRYPTO_METHOD_TLSv1_0_SERVER = 8;

const STREAM_CRYPTO_METHOD_TLSv1_1_CLIENT = 17;

const STREAM_CRYPTO_METHOD_TLSv1_1_SERVER = 16;

const STREAM_CRYPTO_METHOD_TLSv1_2_CLIENT = 33;

const STREAM_CRYPTO_METHOD_TLSv1_2_SERVER = 32;

const STREAM_CRYPTO_METHOD_TLSv1_3_CLIENT = 65;

const STREAM_CRYPTO_METHOD_TLSv1_3_SERVER = 64;

const STREAM_CRYPTO_PROTO_SSLv3 = 4;

const STREAM_CRYPTO_PROTO_TLSv1_0 = 8;

const STREAM_CRYPTO_PROTO_TLSv1_1 = 16;

const STREAM_CRYPTO_PROTO_TLSv1_2 = 32;

const STREAM_CRYPTO_PROTO_TLSv1_3 = 64;

const MT_RAND_MT19937 = 0;

const MT_RAND_PHP = 1;

const LOG_EMERG = 0;

const LOG_ALERT = 1;

const LOG_CRIT = 2;

const LOG_ERR = 3;

const LOG_WARNING = 4;

const LOG_NOTICE = 5;

const LOG_INFO = 6;

const LOG_DEBUG = 7;

const LOG_KERN = 0;

const LOG_USER = 8;

const LOG_MAIL = 16;

const LOG_DAEMON = 24;

const LOG_AUTH = 32;

const LOG_SYSLOG = 40;

const LOG_LPR = 48;

const LOG_NEWS = 56;

const LOG_UUCP = 64;

const LOG_CRON = 72;

const LOG_AUTHPRIV = 80;

const LOG_LOCAL0 = 128;

const LOG_LOCAL1 = 136;

const LOG_LOCAL2 = 144;

const LOG_LOCAL3 = 152;

const LOG_LOCAL4 = 160;

const LOG_LOCAL5 = 168;

const LOG_LOCAL6 = 176;

const LOG_LOCAL7 = 184;

const LOG_PID = 1;

const LOG_CONS = 2;

const LOG_ODELAY = 4;

const LOG_NDELAY = 8;

const LOG_NOWAIT = 16;

const LOG_PERROR = 32;

const DECIMAL_POINT = 65536;

const THOUSANDS_SEP = 65537;

const GROUPING = 65538;

const ERA_YEAR = 131117;

const INT_CURR_SYMBOL = 262144;

const CURRENCY_SYMBOL = 262145;

const MON_DECIMAL_POINT = 262146;

const MON_THOUSANDS_SEP = 262147;

const MON_GROUPING = 262148;

const POSITIVE_SIGN = 262149;

const NEGATIVE_SIGN = 262150;

const INT_FRAC_DIGITS = 262151;

const FRAC_DIGITS = 262152;

const P_CS_PRECEDES = 262153;

const P_SEP_BY_SPACE = 262154;

const N_CS_PRECEDES = 262155;

const N_SEP_BY_SPACE = 262156;

const P_SIGN_POSN = 262157;

const N_SIGN_POSN = 262158;

const PASSWORD_DEFAULT = '2y';

const PASSWORD_BCRYPT_DEFAULT_COST = 12;

const PASSWORD_BCRYPT = '2y';

const PASSWORD_ARGON2I = 'argon2i';

const PASSWORD_ARGON2ID = 'argon2id';

const PASSWORD_ARGON2_DEFAULT_MEMORY_COST = 65536;

const PASSWORD_ARGON2_DEFAULT_TIME_COST = 4;

const PASSWORD_ARGON2_DEFAULT_THREADS = 1;

const PASSWORD_ARGON2_PROVIDER = 'standard';

/**
 * @return array{
 *   'algo': non-empty-string,
 *   'algoName': non-empty-string,
 *   'options': array{'salt'?: int, 'cost'?: int, 'memory_cost'?: int, 'time_cost'?: int, 'threads'?: int, ...},
 * }
 *
 * @no-named-arguments
 * @pure
 */
function password_get_info(string $hash): array
{
}

/**
 * @param array{'salt'?: int, 'cost'?: int, 'memory_cost'?: int, 'time_cost'?: int, 'threads'?: int, ...} $options
 *
 * @return non-empty-string
 *
 * @no-named-arguments
 * @pure
 */
function password_hash(string $password, string|int|null $algo, array $options = []): string
{
}

/**
 * @param array{'salt'?:int, 'cost'?: int, 'memory_cost'?: int, 'time_cost'?: int, 'threads'?: int, ...} $options
 *
 * @no-named-arguments
 * @pure
 */
function password_needs_rehash(string $hash, string|int|null $algo, array $options = []): bool
{
}

/**
 * @no-named-arguments
 * @pure
 */
function password_verify(string $password, string $hash): bool
{
}

/**
 * @return list<non-empty-string>
 *
 * @no-named-arguments
 * @pure
 */
function password_algos(): array
{
}

function dl(string $extension_filename): bool
{
}

function cli_set_process_title(string $title): bool
{
}

/**
 * @pure
 */
function cli_get_process_title(): null|string
{
}

/**
 * @deprecated
 * @pure
 */
function utf8_encode(string $string): string
{
}

/**
 * @pure
 * @deprecated
 */
function utf8_decode(string $string): string
{
}

function error_clear_last(): void
{
}

function sapi_windows_cp_get(string $kind = ''): int
{
}

function sapi_windows_cp_set(int $codepage): bool
{
}

function sapi_windows_cp_conv(int|string $in_codepage, int|string $out_codepage, string $subject): null|string
{
}

function sapi_windows_cp_is_utf8(): bool
{
}

/**
 * @param resource $stream
 */
function sapi_windows_vt100_support($stream, null|bool $enable = null): bool
{
}

function sapi_windows_set_ctrl_handler(null|callable $handler, bool $add = true): bool
{
}

function sapi_windows_generate_ctrl_event(int $event, int $pid = 0): bool
{
}

/**
 * @template TKey
 * @template-covariant TValue
 * @template TSend
 * @template-covariant TReturn
 *
 * @template-implements Traversable<TKey, TValue>
 */
class Generator implements Traversable
{
    /**
     * @return ?TValue
     *
     * @psalm-ignore-nullable-return
     */
    public function current()
    {
    }

    /**
     * @return void
     */
    public function next()
    {
    }

    /**
     * @return TKey
     */
    public function key()
    {
    }

    /**
     * @return bool
     */
    public function valid()
    {
    }

    /**
     * @return void
     */
    public function rewind()
    {
    }

    /**
     * @return TReturn
     */
    public function getReturn()
    {
    }

    /**
     * @param TSend $value
     *
     * @return ?TValue
     *
     * @psalm-ignore-nullable-return
     */
    public function send($value)
    {
    }

    /**
     * @return ?TValue
     *
     * @psalm-ignore-nullable-return
     */
    public function throw(Throwable $exception)
    {
    }
}

function hex2bin(string $string): string|false
{
}

function http_response_code(int $response_code = 0): int|bool
{
}

final class __PHP_Incomplete_Class
{
    /**
     * @var string
     */
    public $__PHP_Incomplete_Class_Name;
}

class php_user_filter
{
    public string $filtername;
    public mixed $params;
    public $stream;

    /**
     * @param resource $in
     * @param resource $out
     * @param int &$consumed
     */
    public function filter($in, $out, &$consumed, bool $closing): int
    {
    }

    public function onCreate(): bool
    {
    }

    public function onClose(): void
    {
    }
}

final class StreamBucket
{
    public $bucket;
    public string $data;
    public int $datalen;
    public int $dataLength;
}

class Directory
{
    public readonly string $path;

    /**
     * @var resource
     */
    public readonly mixed $handle;

    public function close(): void
    {
    }

    public function rewind(): void
    {
    }

    public function read(): string|false
    {
    }
}

/**
 * @throws Error
 *
 * @pure
 */
function constant(string $name): mixed
{
}

/**
 * @pure
 */
function bin2hex(string $string): string
{
}

/**
 * @param int<0, max> $seconds
 */
function sleep(int $seconds): int
{
}

/**
 * @param int<0, max> $microseconds
 */
function usleep(int $microseconds): void
{
}

/**
 * @param positive-int $seconds
 * @param positive-int $nanoseconds
 *
 * @return bool|array{seconds: int, nanoseconds: int}
 */
function time_nanosleep(int $seconds, int $nanoseconds): array|bool
{
}

function time_sleep_until(float $timestamp): bool
{
}

/**
 * @return false|array{tm_sec: int, tm_min: int, tm_hour: int, tm_mday: int, tm_mon: int, tm_year: int, tm_wday: int, tm_yday: int, unparsed: string}
 */
function strptime(string $timestamp, string $format): array|false
{
}

function flush(): void
{
}

/**
 * @pure
 */
function wordwrap(string $string, int $width = 75, string $break = "\n", bool $cut_long_words = false): string
{
}

/**
 * @pure
 */
function htmlspecialchars(
    string $string,
    int $flags = ENT_QUOTES | ENT_SUBSTITUTE,
    null|string $encoding = null,
    bool $double_encode = true,
): string {
}

/**
 * @pure
 */
function htmlentities(
    string $string,
    int $flags = ENT_QUOTES | ENT_SUBSTITUTE,
    null|string $encoding = null,
    bool $double_encode = true,
): string {
}

/**
 * @pure
 */
function html_entity_decode(
    string $string,
    int $flags = ENT_QUOTES | ENT_SUBSTITUTE,
    null|string $encoding = null,
): string {
}

/**
 * @pure
 */
function htmlspecialchars_decode(string $string, int $flags = ENT_QUOTES | ENT_SUBSTITUTE): string
{
}

/**
 * @pure
 */
function get_html_translation_table(
    int $table = 0,
    int $flags = ENT_QUOTES | ENT_SUBSTITUTE,
    string $encoding = 'UTF-8',
): array {
}

/**
 * @pure
 */
function sha1(string $string, bool $binary = false): string
{
}

function sha1_file(string $filename, bool $binary = false): string|false
{
}

/**
 * @pure
 */
function md5(string $string, bool $binary = false): string
{
}

function md5_file(string $filename, bool $binary = false): string|false
{
}

/**
 * @pure
 */
function crc32(string $string): int
{
}

/**
 * @pure
 */
function iptcparse(string $iptc_block): array|false
{
}

function iptcembed(string $iptc_data, string $filename, int $spool = 0): string|bool
{
}

/**
 * @param string $filename
 * @param array &$image_info
 *
 * @return false|array{0: int, 1: int, 2: int, 3: string, bits: int, channels: int, mime: string}
 */
function getimagesize(string $filename, &$image_info): array|false
{
}

/**
 * @pure
 */
function image_type_to_mime_type(int $image_type): string
{
}

/**
 * @pure
 */
function image_type_to_extension(int $image_type, bool $include_dot = true): string|false
{
}

function phpinfo(int $flags = INFO_ALL): bool
{
}

/**
 * @pure
 */
function phpversion(null|string $extension): string|false
{
}

function phpcredits(int $flags = CREDITS_ALL): bool
{
}

/**
 * @return 'cli'|'phpdbg'|'embed'|'apache'|'apache2handler'|'cgi-fcgi'|'cli-server'|'fpm-fcgi'|'litespeed'|false
 * @pure
 */
function php_sapi_name(): string|false
{
}

/**
 * @pure
 */
function php_uname(string $mode = 'a'): string
{
}

/**
 * @pure
 */
function php_ini_scanned_files(): string|false
{
}

/**
 * @pure
 */
function php_ini_loaded_file(): string|false
{
}

/**
 * @pure
 */
function strnatcmp(string $string1, string $string2): int
{
}

/**
 * @pure
 */
function strnatcasecmp(string $string1, string $string2): int
{
}

/**
 * @return int<0,max>
 *
 * @pure
 */
function substr_count(string $haystack, string $needle, int $offset = 0, null|int $length = null): int
{
}

/**
 * @pure
 */
function strspn(string $string, string $characters, int $offset = 0, null|int $length = null): int
{
}

/**
 * @pure
 */
function strcspn(string $string, string $characters, int $offset = 0, null|int $length = null): int
{
}

/**
 * @pure
 */
function strtok(string $string, null|string $token = null): string|false
{
}

/**
 * @pure
 */
function strtoupper(string $string): string
{
}

/**
 * @return lowercase-string
 *
 * @pure
 */
function strtolower(string $string): string
{
}

/**
 * @param int<0, max> $offset
 *
 * @return int<0, max>|false
 *
 * @pure
 */
function strpos(string $haystack, string $needle, int $offset = 0): int|false
{
}

/**
 * @return int<0, max>|false
 *
 * @pure
 */
function stripos(string $haystack, string $needle, int $offset = 0): int|false
{
}

/**
 * @return int<0, max>|false
 *
 * @pure
 */
function strrpos(string $haystack, string $needle, int $offset = 0): int|false
{
}

/**
 * @return int<0, max>|false
 *
 * @pure
 */
function strripos(string $haystack, string $needle, int $offset = 0): int|false
{
}

/**
 * @return ($string is non-empty-string ? non-empty-string : string)
 *
 * @pure
 */
function strrev(string $string): string
{
}

/**
 * @pure
 */
function hebrev(string $string, int $max_chars_per_line = 0): string
{
}

/**
 * @pure
 */
function nl2br(string $string, bool $use_xhtml = true): string
{
}

/**
 * @pure
 *
 * @return ($path is non-empty-string ? non-empty-string : string)
 */
function basename(string $path, string $suffix = ''): string
{
}

/**
 * @pure
 *
 * @return ($path is non-empty-string ? non-empty-string : string)
 */
function dirname(string $path, int $levels = 1): string
{
}

/**
 * @param 1|2|4|8|15 $flags
 *
 * @return ($flags is 15 ? array{dirname?: string, basename: string, extension?: string, filename: string} : string)
 */
function pathinfo(string $path, int $flags = PATHINFO_ALL): array|string
{
}

/**
 * @pure
 */
function stripslashes(string $string): string
{
}

/**
 * @pure
 */
function stripcslashes(string $string): string
{
}

/**
 * @pure
 */
function strstr(string $haystack, string $needle, bool $before_needle = false): string|false
{
}

/**
 * @pure
 */
function stristr(string $haystack, string $needle, bool $before_needle = false): string|false
{
}

/**
 * @pure
 */
function strrchr(string $haystack, string $needle, bool $before_needle = false): string|false
{
}

/**
 * @template T as string
 *
 * @param T $string
 *
 * @return (T is non-empty-string ? non-empty-string : string)
 *
 * @pure
 */
function str_shuffle(string $string): string
{
}

/**
 * @template T as 0|1|2
 *
 * @param T $format
 *
 * @return (T is 0 ? int<0, max> : (
 *   T is 1 ? list<non-empty-string> : (
 *     T is 2 ? array<int<0, max>, non-empty-string> : (
 *       int<0, max>|list<non-empty-string>|array<int<0, max>, non-empty-string>
 *     )
 *   )
 * ))
 *
 * @pure
 */
function str_word_count(string $string, int $format = 0, null|string $characters = null): array|int
{
}

/**
 * @param int<1, max> $length
 *
 * @return list<string>
 *
 * @pure
 */
function str_split(string $string, int $length = 1): array
{
}

/**
 * @pure
 */
function strpbrk(string $string, string $characters): string|false
{
}

/**
 * @pure
 */
function substr_compare(
    string $haystack,
    string $needle,
    int $offset,
    null|int $length = null,
    bool $case_insensitive = false,
): int {
}

/**
 * @pure
 */
function strcoll(string $string1, string $string2): int
{
}

/**
 * @pure
 */
function substr(string $string, int $offset, null|int $length = null): string
{
}

/**
 * @template K as array-key
 *
 * @param string|array<K, string> $string
 * @param string|array<string> $replace
 * @param int|array<int> $offset
 * @param null|int<0, max>|array<int<0, max>> $length
 *
 * @return ($string is string ? string : array<K, string>)
 *
 * @pure
 */
function substr_replace(
    array|string $string,
    array|string $replace,
    array|int $offset,
    array|int|null $length = null,
): array|string {
}

/**
 * @pure
 */
function quotemeta(string $string): string
{
}

/**
 * @pure
 */
function ucfirst(string $string): string
{
}

/**
 * @pure
 */
function lcfirst(string $string): string
{
}

/**
 * @pure
 */
function ucwords(string $string, string $separators = " \t\r\n\f\v"): string
{
}

/**
 * @param string|array<string, string> $from
 *
 * @pure
 */
function strtr(string $string, string|array $from, string $to = ''): string
{
}

/**
 * @pure
 */
function addslashes(string $string): string
{
}

/**
 * @pure
 */
function addcslashes(string $string, string $characters): string
{
}

/**
 * @pure
 */
function rtrim(string $string, string $characters = " \n\r\t\v\0"): string
{
}

/**
 * @param string|array<string> $search
 * @param string|array<string> $replace
 * @param string|array<string> $subject
 *
 * @return ($subject is string ? string : (
 *   $subject is array<string> ? array<string> : string|array<string>
 * ))
 *
 * @pure
 */
function str_replace(
    array|string $search,
    array|string $replace,
    array|string $subject,
    null|int &$count = null,
): array|string {
}

/**
 * @param string|array<string> $search
 * @param string|array<string> $replace
 * @param string|array<string> $subject
 *
 * @return ($subject is string ? string : (
 *   $subject is array<string> ? array<string> : string|array<string>
 * ))
 *
 * @pure
 */
function str_ireplace(
    array|string $search,
    array|string $replace,
    array|string $subject,
    null|int &$count = null,
): array|string {
}

/**
 * @pure
 */
function str_repeat(string $string, int $times): string
{
}

/**
 * @pure
 */
function count_chars(string $string, int $mode = 0): array|string
{
}

/**
 * @pure
 */
function chunk_split(string $string, int $length = 76, string $separator = "\r\n"): string
{
}

/**
 * @pure
 */
function trim(string $string, string $characters = " \n\r\t\v\0"): string
{
}

/**
 * @pure
 */
function ltrim(string $string, string $characters = " \n\r\t\v\0"): string
{
}

/**
 * @param array<string>|string|null $allowed_tags
 *
 * @pure
 */
function strip_tags(string $string, string|array|null $allowed_tags = null): string
{
}

/**
 * @pure
 */
function similar_text(string $string1, string $string2, &$percent = null): int
{
}

/**
 * @return list<non-empty-string>
 *
 * @pure
 */
function explode(string $separator, string $string, int $limit = PHP_INT_MAX): array
{
}

/**
 * @param array<string>|string $separator
 * @param array<string>|null $array
 *
 * @pure
 */
function implode(array|string $separator = '', null|array $array = null): string
{
}

/**
 * @param array<string>|string $separator
 * @param array<string>|null $array
 *
 * @pure
 */
function join(array|string $separator = '', null|array $array = null): string
{
}

/**
 * @param string|array<string>|int $locales
 * @param string|array<string> ...$rest
 */
function setlocale(int $category, string|int|array $locales, string|array ...$rest): string|false
{
}

/**
 * @return array{
 *   decimal_point: string,
 *   thousands_sep: string,
 *   grouping: array<int, int>,
 *   int_curr_symbol: string,
 *   currency_symbol: string,
 *   mon_decimal_point: string,
 *   mon_thousands_sep: string,
 *   mon_grouping: string,
 *   positive_sign: string,
 *   negative_sign: string,
 *   int_frac_digits: string,
 *   frac_digits: string,
 *   p_cs_precedes: bool,
 *   p_sep_by_space: bool,
 *   n_cs_precedes: bool,
 *   n_sep_by_space: bool,
 *   p_sign_posn: int,
 *   n_sign_posn: int
 * }
 *
 * @pure
 */
function localeconv(): array
{
}

/**
 * @template K
 * @template V
 *
 * @param array<K, V> $array
 * @param (callable(V, K): bool) $callback
 *
 * @return V|null
 *
 * @since 8.4
 */
function array_find(array $array, callable $callback): mixed
{
}

/**
 * @template K
 * @template V
 *
 * @param array<K, V> $array
 * @param (callable(V, K): bool) $callback
 *
 * @return K|null
 *
 * @since 8.4
 */
function array_find_key(array $array, callable $callback): mixed
{
}

/**
 * @template K
 * @template V
 *
 * @param array<K, V> $array
 * @param (callable(V, K): bool) $callback
 *
 * @since 8.4
 */
function array_any(array $array, callable $callback): bool
{
}

/**
 * @template K
 * @template V
 *
 * @param array<K, V> $array
 * @param (callable(V, K): bool) $callback
 *
 * @since 8.4
 */
function array_all(array $array, callable $callback): bool
{
}

/**
 * @return null|list<non-empty-string>
 *
 * @since 8.4
 */
function http_get_last_response_headers(): null|array
{
}

/**
 * @since 8.4
 */
function http_clear_last_response_headers(): void
{
}

/**
 * @since 8.4
 * @param array|null $options
 * @return array<int, array>
 */
function request_parse_body(null|array $options = null): array
{
}

function fpow(float $num, float $exponent): float
{
}

enum RoundingMode implements UnitEnum
{
    case HalfAwayFromZero;
    case HalfTowardsZero;
    case HalfEven;
    case HalfOdd;
    case TowardsZero;
    case AwayFromZero;
    case NegativeInfinity;
    case PositiveInfinity;
}

/**
 * @pure
 */
function nl_langinfo(int $item): string|false
{
}

/**
 * @pure
 */
function soundex(string $string): string
{
}

function levenshtein(
    string $string1,
    string $string2,
    int $insertion_cost = 1,
    int $replacement_cost = 1,
    int $deletion_cost = 1,
): int {
}

/**
 * @pure
 */
function chr(int $codepoint): string
{
}

/**
 * @param string $character
 * @return int<0, 255>
 *
 * @pure
 */
function ord(string $character): int
{
}

/**
 * @param-out array<string, string> $result
 *
 * @return void
 */
function parse_str(string $string, &$result): void
{
}

/**
 * @pure
 */
function str_getcsv(string $string, string $separator = ',', string $enclosure = '"', string $escape = "\\"): array
{
}

/**
 * @pure
 */
function str_pad(string $string, int $length, string $pad_string = ' ', int $pad_type = STR_PAD_RIGHT): string
{
}

/**
 * @pure
 */
function chop(string $string, string $characters = " \n\r\t\v\0"): string
{
}

/**
 * @pure
 */
function strchr(string $haystack, string $needle, bool $before_needle = false): string|false
{
}

/**
 * @param string|int|float ...$values
 *
 * @pure
 */
function sprintf(string $format, mixed ...$values): string
{
}

/**
 * @param string|int|float ...$values
 *
 * @return int<0, max>
 */
function printf(string $format, mixed ...$values): int
{
}

/**
 * @pure
 */
function vprintf(string $format, array $values): int
{
}

/**
 * @pure
 */
function vsprintf(string $format, array $values): string
{
}

/**
 * @param resource $stream
 *
 * @pure
 */
function fprintf($stream, string $format, mixed ...$values): int
{
}

/**
 * @param resource $stream
 *
 * @pure
 */
function vfprintf($stream, string $format, array $values): int
{
}

function sscanf(string $string, string $format, mixed &...$vars): array|int|null
{
}

/**
 * @param resource $stream
 */
function fscanf($stream, string $format, mixed &...$vars): array|int|false|null
{
}

/**
 * @pure
 */
function parse_url(string $url, int $component = -1): array|string|int|false|null
{
}

/**
 * @pure
 */
function urlencode(string $string): string
{
}

/**
 * @pure
 */
function urldecode(string $string): string
{
}

/**
 * @pure
 */
function rawurlencode(string $string): string
{
}

/**
 * @pure
 */
function rawurldecode(string $string): string
{
}

/**
 * @pure
 */
function http_build_query(
    object|array $data,
    string $numeric_prefix = '',
    null|string $arg_separator = null,
    int $encoding_type = PHP_QUERY_RFC1738,
): string {
}

function readlink(string $path): string|false
{
}

function linkinfo(string $path): int|false
{
}

function symlink(string $target, string $link): bool
{
}

function link(string $target, string $link): bool
{
}

/**
 * @param null|resource $context
 */
function unlink(string $filename, mixed $context = null): bool
{
}

function exec(string $command, &$output, &$result_code): string|false
{
}

function system(string $command, &$result_code): string|false
{
}

/**
 * @pure
 */
function escapeshellcmd(string $command): string
{
}

/**
 * @pure
 */
function escapeshellarg(string $arg): string
{
}

/**
 * @param-out int $result_code
 *
 * @pure
 */
function passthru(string $command, &$result_code): null|false
{
}

function shell_exec(string $command): string|false|null
{
}

/**
 * @param array<string>|string $command
 * @param array{
 *   0?: resource|array{0: 'pipe', 1: 'r'|'w'}|array{0: 'file', 1: non-empty-string},
 *   1?: resource|array{0: 'pipe', 1: 'r'|'w'}|array{0: 'file', 1: non-empty-string},
 *   2?: resource|array{0: 'pipe', 1: 'r'|'w'}|array{0: 'file', 1: non-empty-string},
 * } $descriptor_spec
 * @param non-empty-string|null $cwd
 * @param null|array<string, string> $env_vars
 * @param null|array{
 *   suppress_errors?: bool,
 *   bypass_shell?: bool,
 *   blocking_pipes?: bool,
 *   create_process_group?: bool,
 *   create_new_console?: bool,
 * } $options
 *
 * @param-out array{
 *   0: resource,
 *   1: resource,
 *   2: resource,
 * } $pipes
 *
 * @return open-resource|false
 */
function proc_open(
    array|string $command,
    array $descriptor_spec,
    null|array &$pipes,
    null|string $cwd = null,
    null|array $env_vars = null,
    null|array $options = null,
) {
}

/**
 * @param resource $process
 */
function proc_close($process): int
{
}

/**
 * @param resource $process
 */
function proc_terminate($process, int $signal = 15): bool
{
}

/**
 * @param resource $process
 *
 * @return array{
 *  'command': string,
 *  'pid': int,
 *  'running': bool,
 *  'signaled': bool,
 *  'stopped': bool,
 *  'exitcode': int,
 *  'termsig': int,
 *  'stopsig': int,
 * }
 */
function proc_get_status($process): array
{
}

function proc_nice(int $priority): bool
{
}

function getservbyname(string $service, string $protocol): int|false
{
}

/**
 * @pure
 */
function getservbyport(int $port, string $protocol): string|false
{
}

/**
 * @pure
 */
function getprotobyname(string $protocol): int|false
{
}

/**
 * @pure
 */
function getprotobynumber(int $protocol): string|false
{
}

/**
 * @pure
 */
function getmyuid(): int|false
{
}

/**
 * @pure
 */
function getmygid(): int|false
{
}

/**
 * @pure
 */
function getmypid(): int|false
{
}

/**
 * @pure
 */
function getmyinode(): int|false
{
}

function getlastmod(): int|false
{
}

/**
 * @pure
 */
function base64_decode(string $string, bool $strict = false): string|false
{
}

/**
 * @pure
 */
function base64_encode(string $string): string
{
}

/**
 * @pure
 */
function convert_uuencode(string $string): string
{
}

/**
 * @pure
 */
function convert_uudecode(string $string): string|false
{
}

/**
 * @return ($num is int ? int : float)
 *
 * @pure
 */
function abs(int|float $num): int|float
{
}

/**
 * @pure
 */
function ceil(int|float $num): float
{
}

/**
 * @pure
 */
function floor(int|float $num): float
{
}

/**
 * @pure
 */
function round(int|float $num, int $precision = 0, RoundingMode|int $mode = 0): float
{
}

/**
 * @pure
 */
function sin(float $num): float
{
}

/**
 * @pure
 */
function cos(float $num): float
{
}

/**
 * @pure
 */
function tan(float $num): float
{
}

/**
 * @pure
 */
function asin(float $num): float
{
}

/**
 * @pure
 */
function acos(float $num): float
{
}

/**
 * @pure
 */
function atan(float $num): float
{
}

/**
 * @pure
 */
function atanh(float $num): float
{
}

/**
 * @pure
 */
function atan2(float $y, float $x): float
{
}

/**
 * @pure
 */
function sinh(float $num): float
{
}

/**
 * @pure
 */
function cosh(float $num): float
{
}

/**
 * @pure
 */
function tanh(float $num): float
{
}

/**
 * @pure
 */
function asinh(float $num): float
{
}

/**
 * @pure
 */
function acosh(float $num): float
{
}

/**
 * @pure
 */
function expm1(float $num): float
{
}

/**
 * @pure
 */
function log1p(float $num): float
{
}

/**
 * @pure
 */
function pi(): float
{
}

/**
 * @pure
 */
function is_finite(float $num): bool
{
}

/**
 * @pure
 */
function is_nan(float $num): bool
{
}

/**
 * @pure
 *
 * @throws DivisionByZeroError
 * @throws ArithmeticError
 */
function intdiv(int $num1, int $num2): int
{
}

/**
 * @pure
 */
function is_infinite(float $num): bool
{
}

/**
 * @pure
 */
function pow(mixed $num, mixed $exponent): object|int|float
{
}

/**
 * @pure
 */
function exp(float $num): float
{
}

/**
 * @pure
 */
function log(float $num, float $base = M_E): float
{
}

/**
 * @pure
 */
function log10(float $num): float
{
}

/**
 * @pure
 */
function sqrt(float $num): float
{
}

/**
 * @pure
 */
function hypot(float $x, float $y): float
{
}

/**
 * @pure
 */
function deg2rad(float $num): float
{
}

/**
 * @pure
 */
function rad2deg(float $num): float
{
}

/**
 * @pure
 */
function bindec(string $binary_string): int|float
{
}

/**
 * @pure
 */
function hexdec(string $hex_string): int|float
{
}

/**
 * @pure
 */
function octdec(string $octal_string): int|float
{
}

/**
 * @pure
 */
function decbin(int $num): string
{
}

/**
 * @pure
 */
function decoct(int $num): string
{
}

/**
 * @pure
 */
function dechex(int $num): string
{
}

/**
 * @pure
 */
function base_convert(string $num, int $from_base, int $to_base): string
{
}

/**
 * @pure
 */
function number_format(
    float $num,
    int $decimals = 0,
    null|string $decimal_separator = '.',
    null|string $thousands_separator = ',',
): string {
}

/**
 * @pure
 */
function fmod(float $num1, float $num2): float
{
}

/**
 * @pure
 */
function fdiv(float $num1, float $num2): float
{
}

/**
 * @pure
 */
function inet_ntop(string $ip): string|false
{
}

/**
 * @pure
 */
function inet_pton(string $ip): string|false
{
}

/**
 * @pure
 */
function ip2long(string $ip): int|false
{
}

/**
 * @pure
 */
function long2ip(int $ip): string
{
}

/**
 * @return ($name is null ? array<string, string> : string|false)
 */
function getenv(null|string $name = null, bool $local_only = false): array|string|false
{
}

function putenv(string $assignment): bool
{
}

/**
 * @param string $short_options
 * @param list<string> $long_options
 *
 * @return array<string, string>|false
 */
function getopt(string $short_options, array $long_options = [], null|int &$rest_index = null): array|false
{
}

function sys_getloadavg(): array|false
{
}

/**
 * @return ($as_float is true ? float : ($as_float is false ? string : string|float))
 */
function microtime(bool $as_float = false): string|float
{
}

/**
 * @return ($as_float is true ? float : (
 *   $ast_float is false ? array{sec: int, usec: int, minuteswest: int, dsttime: int} : array{sec: int, usec: int, minuteswest: int, dsttime: int}|float
 * ))
 */
function gettimeofday(bool $as_float = false): array|float
{
}

/**
 * @return array<string, scalar>|false
 */
function getrusage(int $mode = 0): array|false
{
}

/**
 * @return non-empty-string
 */
function uniqid(string $prefix = '', bool $more_entropy = false): string
{
}

/**
 * @pure
 */
function quoted_printable_decode(string $string): string
{
}

/**
 * @pure
 */
function quoted_printable_encode(string $string): string
{
}

function get_current_user(): string
{
}

function set_time_limit(int $seconds): bool
{
}

/**
 * @pure
 */
function get_cfg_var(string $option): array|string|false
{
}

/**
 * @deprecated
 */
function get_magic_quotes_runtime(): int
{
}

function error_log(
    string $message,
    int $message_type = 0,
    null|string $destination = null,
    null|string $additional_headers = null,
): bool {
}

/**
 * @pure
 */
function boolval(mixed $value): bool
{
}

/**
 * @pure
 */
function intval(mixed $value, int $base = 10): int
{
}

/**
 * @pure
 */
function floatval(mixed $value): float
{
}

/**
 * @pure
 */
function doubleval(mixed $value): float
{
}

function strval(mixed $value): string
{
}

/**
 * @return 'boolean'|'integer'|'double'|'string'|'array'|'object'|'resource'|'NULL'|'unknown type'|'resource (closed)'
 *
 * @pure
 */
function gettype(mixed $value): string
{
}

/**
 * @param 'bool'|'boolean'|'int'|'integer'|'float'|'double'|'string'|'array'|'object'|'null' $type
 */
function settype(mixed &$var, string $type): bool
{
}

/**
 * @assert-if-true null $value
 *
 * @return ($value is null ? true : false)
 *
 * @pure
 */
function is_null(mixed $value): bool
{
}

/**
 * @assert-if-true open-resource $value
 *
 * @return ($value is open-resource ? true : false)
 *
 * @pure
 */
function is_resource(mixed $value): bool
{
}

/**
 * @assert-if-true bool $value
 *
 * @return ($value is bool ? true : false)
 *
 * @pure
 */
function is_bool(mixed $value): bool
{
}

/**
 * @assert-if-true int $value
 *
 * @return ($value is int ? true : false)
 *
 * @pure
 */
function is_long(mixed $value): bool
{
}

/**
 * @assert-if-true float $value
 *
 * @return ($value is float ? true : false)
 *
 * @pure
 */
function is_float(mixed $value): bool
{
}

/**
 * @assert-if-true int $value
 *
 * @return ($value is int ? true : false)
 *
 * @pure
 */
function is_int(mixed $value): bool
{
}

/**
 * @assert-if-true int $value
 *
 * @return ($value is int ? true : false)
 *
 * @pure
 */
function is_integer(mixed $value): bool
{
}

/**
 * @assert-if-true float $value
 *
 * @return ($value is float ? true : false)
 *
 * @pure
 */
function is_double(mixed $value): bool
{
}

/**
 * @assert-if-true float $value
 *
 * @return ($value is float ? true : false)
 *
 * @pure
 * @deprecated
 */
function is_real(mixed $var): bool
{
}

/**
 * @assert-if-true numeric $value
 *
 * @return ($value is numeric ? true : false)
 *
 * @pure
 */
function is_numeric(mixed $value): bool
{
}

/**
 * @assert-if-true string $value
 *
 * @return ($value is string ? true : false)
 *
 * @pure
 */
function is_string(mixed $value): bool
{
}

/**
 * @assert-if-true array<array-key, mixed> $value
 *
 * @return ($value is array ? true : false)
 *
 * @pure
 */
function is_array(mixed $value): bool
{
}

/**
 * @assert-if-true list<mixed> $array
 *
 * @return ($array is =list ? true : false)
 *
 * @pure
 */
function array_is_list(array $array): bool
{
}

/**
 * @assert-if-true iterable $value
 *
 * @return ($values is iterable ? true : false)
 *
 * @pure
 */
function is_iterable(mixed $value): bool
{
}

/**
 * @assert-if-true object $value
 *
 * @return ($value is object ? true : false)
 *
 * @pure
 */
function is_object(mixed $value): bool
{
}

/**
 * @assert-if-true scalar $value
 *
 * @return ($value is scalar ? true : false)
 *
 * @pure
 */
function is_scalar(mixed $value): bool
{
}

/**
 * @param mixed $value
 * @param bool $syntax_only
 *
 * @param-out string $callable_name
 *
 * @assert-if-true callable $value
 *
 * @pure
 */
function is_callable(mixed $value, bool $syntax_only = false, &$callable_name = null): bool
{
}

/**
 * @pure
 */
function is_countable(mixed $value): bool
{
}

/**
 * @param resource $handle
 *
 * @return int<-1, max>
 */
function pclose($handle): int
{
}

/**
 * @return open-resource|false
 */
function popen(string $command, string $mode)
{
}

/**
 * @param null|resource $context
 */
function readfile(string $filename, bool $use_include_path = false, $context = null): int|false
{
}

/**
 * @param resource $stream
 */
function rewind($stream): bool
{
}

/**
 * @param null|resource $context
 */
function rmdir(string $directory, $context = null): bool
{
}

function umask(null|int $mask): int
{
}

/**
 * @param resource $stream
 *
 * @assert-if-true closed-resource $stream
 */
function fclose($stream): bool
{
}

/**
 * @param resource $stream
 */
function feof($stream): bool
{
}

/**
 * @param resource $stream
 */
function fgetc($stream): string|false
{
}

/**
 * @param resource $stream
 */
function fgets($stream, null|int $length = null): string|false
{
}

/**
 * @param resource $stream
 */
function fread($stream, int $length): string|false
{
}

/**
 * @param resource|null $context
 *
 * @return open-resource|false
 *
 * @ignore-falsable-return
 */
function fopen(string $filename, string $mode, bool $use_include_path = false, $context = null)
{
}

/**
 * @param resource $stream
 */
function fpassthru($stream): int
{
}

/**
 * @param resource $stream
 */
function ftruncate($stream, int $size): bool
{
}

/**
 * @param resource $stream
 *
 * @return false|array{
 *   'dev': int<0, max>,
 *   'ino': int<0, max>,
 *   'mode': int<0, max>,
 *   'nlink': int<0, max>,
 *   'uid': int<0, max>,
 *   'gid': int<0, max>,
 *   'rdev': int<0, max>,
 *   'size': int<0, max>,
 *   'atime': int<1750171087, max>,
 *   'mtime': int<1750171087, max>,
 *   'ctime': int<1750171087, max>,
 *   'blksize': int<0, max>,
 *   'blocks': int<0, max>,
 * }
 */
function fstat($stream): array|false
{
}

/**
 * @param resource $stream
 */
function fseek($stream, int $offset, int $whence = SEEK_SET): int
{
}

/**
 * @param resource $stream
 *
 * @return int<0, max>|false
 */
function ftell($stream): int|false
{
}

/**
 * @param resource $stream
 */
function fflush($stream): bool
{
}

/**
 * @param resource $stream
 */
function fsync($stream): bool
{
}

/**
 * @param resource $stream
 */
function fdatasync($stream): bool
{
}

/**
 * @param resource $stream
 *
 * @return int<0, max>|false
 */
function fwrite($stream, string $data, null|int $length = null): int|false
{
}

/**
 * @param resource $stream
 *
 * @return int<0, max>|false
 */
function fputs($stream, string $data, null|int $length = null): int|false
{
}

/**
 * @param null|resource $context
 */
function mkdir(string $directory, int $permissions = 0777, bool $recursive = false, $context = null): bool
{
}

/**
 * @param null|resource $context
 */
function rename(string $from, string $to, $context = null): bool
{
}

/**
 * @param null|resource $context
 */
function copy(string $from, string $to, $context = null): bool
{
}

function tempnam(string $directory, string $prefix): string|false
{
}

/**
 * @return resource|false
 */
function tmpfile()
{
}

/**
 * @param null|resource $context
 *
 * @return array<int, string>|false
 */
function file(string $filename, int $flags = 0, $context = null): array|false
{
}

/**
 * @param null|resource $context
 */
function file_get_contents(
    string $filename,
    bool $use_include_path = false,
    $context = null,
    int $offset = 0,
    null|int $length = null,
): string|false {
}

/**
 * @param null|resource $context
 *
 * @return int<0, max>|false
 */
function file_put_contents(string $filename, mixed $data, int $flags = 0, $context = null): int|false
{
}

/**
 * @return array{type: int, message: string, file: string, line: int}|null
 *
 * @pure
 */
function error_get_last(): null|array
{
}

/**
 * @tempalte I
 * @template R
 *
 * @template (callble(...I): R) $callback
 *
 * @param I ...$args
 *
 * @return R
 */
function call_user_func(callable $callback, mixed ...$args): mixed
{
}

/**
 * @tempalte I
 * @template R
 *
 * @template (callble(...I): R) $callback
 *
 * @param array<I> $args
 *
 * @return R
 */
function call_user_func_array(callable $callback, array $args): mixed
{
}

/**
 * @tempalte I
 * @template R
 *
 * @template (callble(...I): R) $callback
 *
 * @param I ...$args
 *
 * @return R
 */
function forward_static_call(callable $callback, mixed ...$args): mixed
{
}

/**
 * @tempalte I
 * @template R
 *
 * @template (callble(...I): R) $callback
 *
 * @param array<I> $args
 *
 * @return R
 */
function forward_static_call_array(callable $callback, array $args): mixed
{
}

/**
 * @return non-empty-string
 */
function serialize(mixed $value): string
{
}

function unserialize(string $data, array $options = []): mixed
{
}

function var_dump(mixed $value, mixed ...$values): void
{
}

/**
 * @return ($return is true ? non-empty-string : null)
 */
function var_export(mixed $value, bool $return = false): null|string
{
}

function debug_zval_dump(mixed $value, mixed ...$values): void
{
}

/**
 * @return ($return is true ? non-empty-string : bool)
 */
function print_r(mixed $value, bool $return = false): string|bool
{
}

function memory_get_usage(bool $real_usage = false): int
{
}

function memory_get_peak_usage(bool $real_usage = false): int
{
}

function memory_reset_peak_usage(): void
{
}

/**
 * @template I
 *
 * @param callable(...I) $callback
 * @param I ...$args
 *
 * @return bool|null
 */
function register_shutdown_function(callable $callback, mixed ...$args): void
{
}

/**
 * @template I
 *
 * @param callable(...I) $callback
 * @param I ...$args
 */
function register_tick_function(callable $callback, mixed ...$args): bool
{
}

function unregister_tick_function(callable $callback): void
{
}

/**
 * @return ($return is true ? string|false : bool)
 */
function highlight_file(string $filename, bool $return = false): string|bool
{
}

/**
 * @return ($return is true ? string|false : bool)
 */
function show_source(string $filename, bool $return = false): string|bool
{
}

/**
 * @return ($return is true ? string|false : bool)
 */
function highlight_string(string $string, bool $return = false): string|bool
{
}

/**
 * @return ($as_number is true ? int|float|false : list{int, int}|false)
 *
 * @mutation-free
 */
function hrtime(bool $as_number = false): array|int|float|false
{
}

function php_strip_whitespace(string $filename): string
{
}

function ini_get(string $option): string|false
{
}

/**
 * @return array{'global_value': string, 'local_value': string, 'access': int}|false
 */
function ini_get_all(null|string $extension, bool $details = true): array|false
{
}

function ini_set(string $option, string|int|float|bool|null $value): string|false
{
}

function ini_alter(string $option, string|int|float|bool|null $value): string|false
{
}

function ini_restore(string $option): void
{
}

function ini_parse_quantity(string $shorthand): int
{
}

function get_include_path(): string|false
{
}

function set_include_path(string $include_path): string|false
{
}

function setcookie(
    string $name,
    string $value = '',
    int $expires_or_options = 0,
    string $path = '',
    string $domain = '',
    bool $secure = false,
    bool $httponly = false,
): bool {
}

function setcookie(string $name, string $value = '', array $options = []): bool
{
}

/**
 * @param string $value
 * @param int $expires_or_options
 * @param string $path
 * @param string $domain
 * @param bool $secure
 * @param bool $httponly
 */
function setrawcookie(
    string $name,
    $value = '',
    $expires_or_options = 0,
    $path = '',
    $domain = '',
    $secure = false,
    $httponly = false,
): bool {
}

/**
 * @param string $value
 */
function setrawcookie(string $name, $value = '', array $options = []): bool
{
}

function header(string $header, bool $replace = true, int $response_code = 0): void
{
}

function header_remove(null|string $name = null): void
{
}

/**
 * @param-out string $filename
 * @param-out int $line
 */
function headers_sent(&$filename = null, &$line = null): bool
{
}

/**
 * @return list<string>
 */
function headers_list(): array
{
}

/**
 * @return array<string, string>|false
 */
function apache_request_headers(): false|array
{
}

/**
 * @return array<string, string>|false
 */
function getallheaders(): false|array
{
}

/**
 * @return 0|1
 */
function connection_aborted(): int
{
}

function connection_status(): int
{
}

function ignore_user_abort(null|bool $enable = null): int
{
}

function parse_ini_file(
    string $filename,
    bool $process_sections = false,
    int $scanner_mode = INI_SCANNER_NORMAL,
): array|false {
}

function parse_ini_string(
    string $ini_string,
    bool $process_sections = false,
    int $scanner_mode = INI_SCANNER_NORMAL,
): array|false {
}

function is_uploaded_file(string $filename): bool
{
}

function move_uploaded_file(string $from, string $to): bool
{
}

/**
 * @return false|array{
 *   'description': string,
 *   'mac': string,
 *   'mtu': int,
 *   'unicast': list<string>,
 *   'up': bool,
 * }
 */
function net_get_interfaces(): array|false
{
}

function gethostbyaddr(string $ip): string|false
{
}

function gethostbyname(string $hostname): string
{
}

/**
 * @return list<non-empty-string>|false
 */
function gethostbynamel(string $hostname): array|false
{
}

/**
 * @return non-empty-string|false
 */
function gethostname(): string|false
{
}

function dns_check_record(string $hostname, string $type = 'MX'): bool
{
}

function checkdnsrr(string $hostname, string $type = 'MX'): bool
{
}

/**
 * @param-out list<non-empty-string> $hosts
 * @param-out array $weights
 */
function dns_get_mx(string $hostname, &$hosts, &$weights = null): bool
{
}

/**
 * @param-out list<non-empty-string> $hosts
 * @param-out array $weights
 */
function getmxrr(string $hostname, &$hosts, &$weights = null): bool
{
}

/**
 * @param-out array $authoritative_name_servers
 * @param-out array $additional_records
 */
function dns_get_record(
    string $hostname,
    int $type = DNS_ANY,
    &$authoritative_name_servers = null,
    &$additional_records = null,
    bool $raw = false,
): array|false {
}

/**
 * @template R of null|array<array-key, resource>
 * @template W of null|array<array-key, resource>
 * @template E of null|array<array-key, resource>
 *
 * @param R $read
 * @param W $write
 * @param E $except
 *
 * @param-out (R is null ? null : array<array-key, resource>) $read
 * @param-out (W is null ? null : array<array-key, resource>) $write
 * @param-out (E is null ? null : array<array-key, resource>) $except
 *
 * @return false|int<0, max>
 */
function stream_select(
    null|array &$read,
    null|array &$write,
    null|array &$except,
    null|int $seconds,
    null|int $microseconds,
): int|false {
}

/**
 * @return resource
 */
function stream_context_create(null|array $options = null, null|array $params = null): mixed
{
}

/**
 * @param resource $context
 */
function stream_context_set_params($context, array $params): bool
{
}

/**
 * @param resource $context
 *
 * @return array{notification: string, options: array}
 */
function stream_context_get_params($context): array
{
}

/**
 * @param resource $context
 */
function stream_context_set_option($context, string $wrapper_or_options, string $option_name, mixed $value): bool
{
}

/**
 * @param resource $stream_or_context
 */
function stream_context_set_option($stream_or_context, array $options): bool
{
}

/**
 * @param resource $context
 */
function stream_context_set_options($context, array $options): bool
{
}

/**
 * @param resource $stream_or_context
 */
function stream_context_get_options($stream_or_context): array
{
}

/**
 * @return resource
 */
function stream_context_get_default(null|array $options)
{
}

/**
 * @return resource
 */
function stream_context_set_default(array $options)
{
}

/**
 * @param resource $stream
 *
 * @return resource
 */
function stream_filter_prepend($stream, string $filter_name, int $mode = 0, mixed $params = null)
{
}

/**
 * @param resource $stream
 *
 * @return resource|false
 */
function stream_filter_append($stream, string $filter_name, int $mode = 0, mixed $params = null)
{
}

/**
 * @param resource $stream_filter
 */
function stream_filter_remove($stream_filter): bool
{
}

/**
 * @param null|resource $context
 *
 * @param-out null|int $error_code
 * @param-out null|string $error_message
 *
 * @return resource|false
 */
function stream_socket_client(
    string $address,
    &$error_code = null,
    &$error_message = null,
    null|float $timeout = null,
    int $flags = STREAM_CLIENT_CONNECT,
    $context = null,
) {
}

/**
 * @param null|resource $context
 *
 * @param-out null|int $error_code
 * @param-out null|string $error_message
 *
 * @return resource|false
 */
function stream_socket_server(
    string $address,
    &$error_code = null,
    &$error_message = null,
    int $flags = STREAM_SERVER_BIND | STREAM_SERVER_LISTEN,
    $context = null,
) {
}

/**
 * @param resource $socket
 *
 * @param-out string $peer_name
 *
 * @return resource|false
 */
function stream_socket_accept($socket, null|float $timeout = null, &$peer_name = null)
{
}

/**
 * @param resource $socket
 */
function stream_socket_get_name($socket, bool $remote): string|false
{
}

/**
 * @param resource $socket
 *
 * @param-out string $address
 */
function stream_socket_recvfrom($socket, int $length, int $flags = 0, &$address): string|false
{
}

/**
 * @param resource $socket
 */
function stream_socket_sendto($socket, string $data, int $flags = 0, string $address = ''): int|false
{
}

/**
 * @param resource $stream
 * @param null|resource $session_stream
 */
function stream_socket_enable_crypto(
    $stream,
    bool $enable,
    null|int $crypto_method = null,
    $session_stream = null,
): int|bool {
}

/**
 * @param resource $stream
 */
function stream_socket_shutdown($stream, int $mode): bool
{
}

/**
 * @return list{resource, resource}|false
 */
function stream_socket_pair(int $domain, int $type, int $protocol): array|false
{
}

/**
 * @param resource $from
 * @param resource $to
 */
function stream_copy_to_stream($from, $to, null|int $length, int $offset = 0): int|false
{
}

/**
 * @param resource $stream
 */
function stream_get_contents($stream, null|int $length = null, int $offset = -1): string|false
{
}

/**
 * @param resource $stream
 */
function stream_supports_lock($stream): bool
{
}

/**
 * @param resource $stream
 */
function fgetcsv(
    $stream,
    null|int $length = null,
    string $separator = ',',
    string $enclosure = '"',
    string $escape = '\\',
): array|false {
}

/**
 * @param resource $stream
 */
function fputcsv(
    $stream,
    array $fields,
    string $separator = ',',
    string $enclosure = '"',
    string $escape = "\\",
    string $eol = PHP_EOL,
): int|false {
}

/**
 * @param resource $stream
 * @param-out int $would_block
 *
 * @return bool
 */
function flock($stream, int $operation, &$would_block = null): bool
{
}

/**
 * @return array<string, string>|false
 */
function get_meta_tags(string $filename, bool $use_include_path = false): array|false
{
}

/**
 * @param resource $stream
 */
function stream_set_write_buffer($stream, int $size): int
{
}

/**
 * @param resource $stream
 */
function stream_set_read_buffer($stream, int $size): int
{
}

/**
 * @param resource $stream
 */
function set_file_buffer($stream, int $size): int
{
}

/**
 * @param resource $stream
 */
function stream_set_blocking($stream, bool $enable): bool
{
}

/**
 * @param resource $stream
 */
function socket_set_blocking($stream, bool $enable): bool
{
}

/**
 * @param resource $stream
 *
 * @return array{
 *   timed_out: bool,
 *   blocked: bool,
 *   eof: bool,
 *   unread_bytes: int,
 *   stream_type: string,
 *   wrapper_type?: string,
 *   wrapper_data: mixed,
 *   mode: string,
 *   seekable: bool,
 *   uri: string,
 *   crypto: array,
 *   mediatype: string,
 * }
 */
function stream_get_meta_data($stream): array
{
}

/**
 * @param resource $stream
 */
function stream_get_line($stream, int $length, string $ending = ''): string|false
{
}

function stream_wrapper_register(string $protocol, string $class, int $flags = 0): bool
{
}

function stream_register_wrapper(string $protocol, string $class, int $flags = 0): bool
{
}

function stream_resolve_include_path(string $filename): string|false
{
}

function stream_wrapper_unregister(string $protocol): bool
{
}

function stream_wrapper_restore(string $protocol): bool
{
}

/**
 * @return list<string>
 */
function stream_get_wrappers(): array
{
}

/**
 * @return list<string>
 */
function stream_get_transports(): array
{
}

/**
 * @param string|resource $stream
 */
function stream_is_local($stream): bool
{
}

/**
 * @param null|resource $context
 */
function get_headers(string $url, bool $associative = false, $context = null): array|false
{
}

/**
 * @param resource $stream
 */
function stream_set_timeout($stream, int $seconds, int $microseconds = 0): bool
{
}

/**
 * @param resource $stream
 */
function socket_set_timeout($stream, int $seconds, int $microseconds = 0): bool
{
}

/**
 * @param resource $stream
 */
function socket_get_status($stream): array
{
}

function realpath(string $path): string|false
{
}

function fnmatch(string $pattern, string $filename, int $flags = 0): bool
{
}

/**
 * @param string $hostname <p>
 * @param int $port
 *
 * @param float|null $timeout
 *
 * @param-out int $error_code
 * @param-out string $error_message
 *
 * @return resource|false
 */
function fsockopen(
    string $hostname,
    int $port = -1,
    &$error_code = null,
    &$error_message = null,
    null|float $timeout = null,
) {
}

/**
 * @param string $hostname
 * @param int $port
 * @param float|null $timeout
 *
 * @param-out int $error_code
 * @param-out string $error_message
 *
 * @return resource|false
 */
function pfsockopen(
    string $hostname,
    int $port = -1,
    &$error_code = null,
    &$error_message = null,
    null|float $timeout = null,
) {
}

/**
 * @pure
 */
function pack(string $format, mixed ...$values): string
{
}

/**
 * @return ($format is 'a'|'A'|'h'|'H' ? array{1: string}|false : (
 *   $format is 'c' ? array{1: int<-128, 127>}|false : (
 *     $format is 'C' ? array{1: int<0, 255>}|false : (
 *       $format is 's' ? array{1: int<-32768, 32767>}|false : (
 *         $format is 'S'|'n'|'v' ? array{1: int<0, 65535>}|false : (
 *           $format is 'l' ? array{1: int<-2147483648, 2147483647>}|false : (
 *             $format is 'L'|'N'|'V' ? array{1: int<0, 4294967295>}|false : (
 *               $format is 'q'|'Q'|'J'|'P' ? array{1: int}|false : (
 *                 $format is 'f'|'g'|'G'|'d'|'e'|'E' ? array{1: float}|false : (
 *                   array<int>|false
 *                 )
 *               )
 *             )
 *           )
 *         )
 *       )
 *     )
 *   )
 * ))
 *
 * @pure
 */
function unpack(string $format, string $string, int $offset = 0): array|false
{
}

function get_browser(null|string $user_agent, bool $return_array = false): object|array|false
{
}

/**
 * @pure
 */
function crypt(string $string, string $salt): string
{
}

/**
 * @param null|resource $context
 *
 * @return resource|false
 */
function opendir(string $directory, $context = null)
{
}

/**
 * @param null|resource $dir_handle
 */
function closedir($dir_handle = null): void
{
}

function chdir(string $directory): bool
{
}

function chroot(string $directory): bool
{
}

/**
 * @return non-empty-string|false
 */
function getcwd(): string|false
{
}

/**
 * @param resource $dir_handle
 */
function rewinddir($dir_handle): void
{
}

/**
 * @param resource $dir_handle
 *
 * @return non-empty-string|false
 */
function readdir($dir_handle): string|false
{
}

/**
 * @param resource $context
 */
function dir(string $directory, $context): Directory|false
{
}

/**
 * @param resource $context
 */
function getdir(string $directory, $context = null): Directory|false
{
}

/**
 * @param resource|null $context
 *
 * @return list<non-empty-string>|false
 */
function scandir(string $directory, int $sorting_order = 0, $context = null): array|false
{
}

/**
 * @return list<non-empty-string>|false
 */
function glob(string $pattern, int $flags = 0): array|false
{
}

/**
 * @return int<1750595956, max>|false
 */
function fileatime(string $filename): int|false
{
}

/**
 * @return int<1750595956, max>|false
 */
function filectime(string $filename): int|false
{
}

/**
 * @return int<0, max>|false
 */
function filegroup(string $filename): int|false
{
}

/**
 * @return int<0, max>|false
 */
function fileinode(string $filename): int|false
{
}

/**
 * @return int<1750595956, max>|false
 */
function filemtime(string $filename): int|false
{
}

/**
 * @return int<0, max>|false
 */
function fileowner(string $filename): int|false
{
}

/**
 * @return int<0, max>|false
 */
function fileperms(string $filename): int|false
{
}

/**
 * @return int<0, max>|false
 */
function filesize(string $filename): int|false
{
}

/**
 * @return 'fifo'|'char'|'dir'|'block'|'link'|'file'|'socket'|'unknown'|false
 */
function filetype(string $filename): string|false
{
}

function file_exists(string $filename): bool
{
}

function is_writable(string $filename): bool
{
}

function is_writeable(string $filename): bool
{
}

function is_readable(string $filename): bool
{
}

function is_executable(string $filename): bool
{
}

function is_file(string $filename): bool
{
}

function is_dir(string $filename): bool
{
}

function is_link(string $filename): bool
{
}

/**
 * @return array{
 *   'dev': int,
 *   'ino': int,
 *   'mode': int,
 *   'nlink': int,
 *   'uid': int,
 *   'gid': int,
 *   'rdev': int,
 *   'size': int,
 *   'atime': int,
 *   'mtime': int,
 *   'ctime': int,
 *   'blksize': int,
 *   'blocks': int,
 * }|false
 */
function stat(string $filename): array|false
{
}

/**
 * @return array{
 *   'dev': int,
 *   'ino': int,
 *   'mode': int,
 *   'nlink': int,
 *   'uid': int,
 *   'gid': int,
 *   'rdev': int,
 *   'size': int,
 *   'atime': int,
 *   'mtime': int,
 *   'ctime': int,
 *   'blksize': int,
 *   'blocks': int,
 * }|false
 */
function lstat(string $filename): array|false
{
}

function chown(string $filename, string|int $user): bool
{
}

function chgrp(string $filename, string|int $group): bool
{
}

function lchown(string $filename, string|int $user): bool
{
}

function lchgrp(string $filename, string|int $group): bool
{
}

function chmod(string $filename, int $permissions): bool
{
}

function touch(string $filename, null|int $mtime = null, null|int $atime = null): bool
{
}

function clearstatcache(bool $clear_realpath_cache = false, string $filename = ''): void
{
}

function disk_total_space(string $directory): float|false
{
}

function disk_free_space(string $directory): float|false
{
}

function diskfreespace(string $directory): float|false
{
}

/**
 * @param string|array<string, string> $additional_headers
 */
function mail(
    string $to,
    string $subject,
    string $message,
    array|string $additional_headers = [],
    string $additional_params = '',
): bool {
}

function openlog(string $prefix, int $flags, int $facility): bool
{
}

const ARRAY_FILTER_USE_BOTH = 1;

const ARRAY_FILTER_USE_KEY = 2;

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> ...$arrays
 *
 * @return array<K, V>
 *
 * @no-named-arguments
 * @pure
 */
function array_merge_recursive(array ...$arrays)
{
}

/**
 * @param array<array-key, mixed> $array
 * @param array<array-key, mixed> ...$replacements
 *
 * @return array<array-key, mixed>
 *
 * @no-named-arguments
 * @pure
 */
function array_replace(array $array, array ...$replacements): array
{
}

/**
 * @param array<array-key, mixed> $array
 * @param array<array-key, mixed> ...$replacements
 *
 * @return array<array-key, mixed>
 *
 * @no-named-arguments
 * @pure
 */
function array_replace_recursive(array $array, array ...$replacements): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param V $filter_value
 * @param bool $strict
 *
 * @return ($array is non-empty-array|non-empty-list ? non-empty-list<K> : list<K>)
 *
 * @no-named-arguments
 * @pure
 */
function array_keys(array $array, mixed $filter_value = null, bool $strict = false): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @return ($array is non-empty-array|non-empty-list ? non-empty-list<V> : list<V>)
 *
 * @no-named-arguments
 * @pure
 */
function array_values(array $array): array
{
}

/**
 * @template K as array-key
 * @template V as array-key
 *
 * @param array<K, V> $array
 *
 * @return array<V, int>
 *
 * @no-named-arguments
 * @pure
 */
function array_count_values(array $array): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<array-key, array<K, V>|object>|list<array<K,V>|object> $array
 * @param K|null $column_key
 * @param K|null $index_key
 *
 * @return array<array-key, V>
 *
 * @no-named-arguments
 * @pure
 */
function array_column(array $array, string|int|null $column_key, string|int|null $index_key = null): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param bool $preserve_keys
 *
 * @return ($preserve_keys is true ? array<K, V> : list<V>)
 *
 * @no-named-arguments
 * @pure
 */
function array_reverse(array $array, bool $preserve_keys = false): array
{
}

/**
 * @template K as array-key
 * @template V
 * @template I
 *
 * @param array<K, V> $array
 * @param (callable(null|I, V): I) $callback
 * @param null|I $initial
 *
 * @return I
 */
function array_reduce(array $array, callable $callback, mixed $initial = null): mixed
{
}

/**
 * @template K as array-key
 * @template V
 * @template T
 *
 * @param array<K, V> $array
 * @param T $value
 *
 * @return ($length is int<1, max> ? non-empty-array<K, V|T> : array<K, V|T>)
 *
 * @pure
 */
function array_pad(array $array, int $length, mixed $value): array
{
}

/**
 * @template K as array-key
 * @template V as array-key
 *
 * @param array<K, V> $array
 *
 * @return ($array is non-empty-array|non-empty-list ? non-empty-array<V, K> : array<V, K>)
 *
 * @pure
 */
function array_flip(array $array): array
{
}

/**
 *
 * @template V
 *
 * @param array<string, V> $array
 * @param int $case
 *
 * @return ($array is non-empty-array|non-empty-list ? non-empty-array<string, V> : array<string, V>)
 *
 * @pure
 */
function array_change_key_case(array $array, int $case = CASE_LOWER): array
{
}

/**
 * @template K as array-key
 *
 * @param array<K, mixed> $array
 * @param int<0, max> $num
 *
 * @return ($num is 1 ? K : array<K>)
 *
 * @pure
 */
function array_rand(array $array, int $num = 1): array|string|int
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param int<0, 5> $flags
 *
 * @return array<K, V>
 *
 * @pure
 */
function array_unique(array $array, int $flags = SORT_STRING): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param array ...$arrays
 *
 * @return array<K, V>
 * @pure
 */
function array_intersect(array $array, array ...$arrays): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param array ...$arrays
 *
 * @return array<K, V>
 *
 * @pure
 */
function array_intersect_key(array $array, array ...$arrays): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param array ...$arrays
 *
 * @return array<K, V>
 * @pure
 */
function array_intersect_assoc(array $array, array ...$arrays): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param array ...$arrays
 *
 * @return array<K, V>
 * @pure
 */
function array_diff(array $array, array ...$arrays): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param array ...$arrays
 *
 * @return array<K, V>
 * @pure
 */
function array_diff_key(array $array, array ...$arrays): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param array ...$arrays
 *
 * @return array<K, V>
 *
 * @pure
 */
function array_diff_assoc(array $array, array ...$arrays): array
{
}

/**
 * @param array<int|float> $array
 *
 * @return ($array is array<int> ? int : float)
 *
 * @pure
 */
function array_sum(array $array): int|float
{
}

/**
 * @param array<int|float> $array
 *
 * @pure
 */
function array_product(array $array): int|float
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param null|(callable(V, K): bool)|(callable(V): bool)|(callable(K): bool) $callback
 *
 * @return array<K, V>
 */
function array_filter(array $array, null|callable $callback = null, int $mode = 0): array
{
}

/**
 * @template K as array-key
 * @template V
 * @template S
 * @template U
 *
 * @param (callable(V): U)|(callable(V, S): U)|null $callback
 * @param array<K, V> $array
 * @param array<S> ...$arrays
 *
 * @return ($array is list<V> ? list<U> : array<K, U>)
 */
function array_map(null|callable $callback, array $array, array ...$arrays): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @return ($preserve_keys is true ? list<array<K, V>> : list<list<V>>)
 *
 * @pure
 */
function array_chunk(array $array, int $length, bool $preserve_keys = false): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K> $keys
 * @param array<V> $values
 *
 * @return ($keys is non-empty-array|non-empty-list ? non-empty-array<K, V> : array<K, V>)
 *
 * @pure
 */
function array_combine(array $keys, array $values): array
{
}

/**
 * @pure
 */
function array_key_exists(string|int|float|bool|null $key, array $array): bool
{
}

/**
 * @template K as array-key
 *
 * @param array<K, mixed> $array
 *
 * @return ($array is non-empty-array|non-empty-list ? K : null)
 *
 * @pure
 */
function array_key_first(array $array): string|int|null
{
}

/**
 * @template V
 *
 * @param array<array-key, V> $array
 *
 * @return ($array is non-empty-array|non-empty-list ? V : null)
 *
 * @pure
 */
function array_first(array $array): mixed
{
}

/**
 * @template K as array-key
 *
 * @param array<K, mixed> $array
 *
 * @return ($array is non-empty-array|non-empty-list ? K : null)
 *
 * @pure
 */
function array_key_last(array $array): string|int|null
{
}

/**
 * @template V
 *
 * @param array<array-key, V> $array
 *
 * @return ($array is non-empty-array|non-empty-list ? V : null)
 *
 * @pure
 */
function array_last(array $array): mixed
{
}

/**
 * @pure
 */
function pos(object|array $array): mixed
{
}

/**
 * @return ($value is non-empty-array|non-empty-list ? int<1, max> : int<0, max>)
 *
 * @pure
 */
function sizeof(Countable|array $value, int $mode = COUNT_NORMAL): int
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param K $key
 * @param array<K, V> $array
 */
function key_exists($key, array $array): bool
{
}

/**
 * @assert truthy $assertion
 */
function assert(mixed $assertion, Throwable|string|null $description = null): bool
{
}

class AssertionError extends Error
{
}

/**
 * @deprecated
 */
function assert_options(int $option, mixed $value): mixed
{
}

/**
 * @param null|'<'|'lt'|'<='|'le'|'>'|'gt'|'>='|'ge'|'=='|'='|'eq'|'!='|'<>'|'ne' $operator
 *
 * @return ($operator is null ? int : bool)
 *
 * @pure
 */
function version_compare(string $version1, string $version2, null|string $operator = null): int|bool
{
}

function ftok(string $filename, string $project_id): int
{
}

/**
 * @pure
 */
function str_rot13(string $string): string
{
}

/**
 * @return list<string>
 */
function stream_get_filters(): array
{
}

/**
 * @param resource $stream
 */
function stream_isatty($stream): bool
{
}

/**
 * @param class-string $class
 */
function stream_filter_register(string $filter_name, string $class): bool
{
}

/**
 * @param resource $brigade
 */
function stream_bucket_make_writeable($brigade): StreamBucket|null
{
}

/**
 * @param resource $brigade
 */
function stream_bucket_prepend($brigade, StreamBucket $bucket): void
{
}

/**
 * @param resource $brigade
 */
function stream_bucket_append($brigade, StreamBucket $bucket): void
{
}

/**
 * @param resource $stream
 */
function stream_bucket_new($stream, string $buffer): StreamBucket
{
}

function output_add_rewrite_var(string $name, string $value): bool
{
}

function output_reset_rewrite_vars(): bool
{
}

/**
 * @return non-empty-string
 */
function sys_get_temp_dir(): string
{
}

function realpath_cache_get(): array
{
}

function realpath_cache_size(): int
{
}

function get_mangled_object_vars(object $object): array
{
}

/**
 * @return non-empty-string
 *
 * @pure
 */
function get_debug_type(mixed $value): string
{
}

/**
 * @param resource $resource
 *
 * @pure
 */
function get_resource_id($resource): int
{
}

/**
 * @no-named-arguments
 */
function array_diff_ukey(array $array, ...$rest): array
{
}

/**
 * @no-named-arguments
 */
function array_udiff(array $array, ...$rest): array
{
}

/**
 * @no-named-arguments
 */
function array_udiff_assoc(array $array, ...$rest): array
{
}

/**
 * @no-named-arguments
 */
function array_diff_uassoc(array $array, ...$rest): array
{
}

/**
 * @no-named-arguments
 */
function array_udiff_uassoc(array $array, ...$rest): array
{
}

/**
 * @no-named-arguments
 */
function array_uintersect_assoc(array $array, ...$rest): array
{
}

/**
 * @no-named-arguments
 */
function array_intersect_uassoc(array $array, ...$rest): array
{
}

/**
 * @no-named-arguments
 */
function array_uintersect_uassoc(array $array, ...$rest): array
{
}

/**
 * @no-named-arguments
 */
function array_intersect_ukey(array $array, ...$rest): array
{
}

/**
 * @no-named-arguments
 */
function array_uintersect(array $array, ...$rest): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @param-out array<K, V> $array
 */
function ksort(array &$array, int $flags = SORT_REGULAR): true
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @param-out array<K, V> $array
 */
function krsort(array &$array, int $flags = SORT_REGULAR): true
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @param-out array<K, V> $array
 */
function natsort(array &$array): true
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @param-out array<K, V> $array
 */
function natcasesort(array &$array): true
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @param-out array<K, V> $array
 */
function asort(array &$array, int $flags = SORT_REGULAR): true
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @param-out array<K, V> $array
 */
function arsort(array &$array, int $flags = SORT_REGULAR): true
{
}

/**
 * @template T
 *
 * @param array<T> $array
 *
 * @param-out list<T> $array
 */
function sort(array &$array, int $flags = SORT_REGULAR): true
{
}

/**
 * @template T
 *
 * @param array<T> $array
 *
 * @param-out list<T> $array
 */
function rsort(array &$array, int $flags = SORT_REGULAR): true
{
}

/**
 * @template T
 *
 * @param array<T> $array
 * @param (callable(T, T): int) $callback
 *
 * @param-out list<T> $array
 */
function usort(array &$array, callable $callback): true
{
}

/**
 * @template K
 * @template V
 *
 * @param array<K, V> $array
 * @param (callable(V, V): int) $callback
 *
 * @param-out array<K, V> $array
 */
function uasort(array &$array, callable $callback): true
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param (callable(K, K): int) $callback
 *
 * @param-out array<K, V> $array
 */
function uksort(array &$array, callable $callback): true
{
}

/**
 * @template T
 *
 * @param array<T> $array
 *
 * @param-out list<T> $array
 */
function shuffle(array &$array): true
{
}

/**
 * @return ($value is non-empty-array|non-empty-list ? int<1, max> : int<0, max>)
 *
 * @pure
 */
function count(Countable|array $value, int $mode = COUNT_NORMAL): int
{
}

/**
 * @template T
 *
 * @param object|array<T> $array
 *
 * @return ($array is non-empty-array|non-empty-list ? T : false)
 */
function end(object|array &$array): mixed
{
}

/**
 * @template T
 *
 * @param object|array<T> $array
 *
 * @return T|false
 */
function prev(object|array &$array): mixed
{
}

/**
 * @template T
 *
 * @param object|array<T> $array
 *
 * @return T|false
 */
function next(object|array &$array): mixed
{
}

/**
 * @template T
 *
 * @param object|array<T> $array
 *
 * @return T|false
 */
function reset(object|array &$array): mixed
{
}

/**
 * @template T
 *
 * @param object|array<T> $array
 *
 * @return T|null
 *
 * @pure
 */
function current(object|array $array): mixed
{
}

/**
 * @template K of array-key
 *
 * @param object|array<K, mixed> $array
 *
 * @return K|null
 *
 * @pure
 */
function key(object|array $array): string|int|null
{
}

/**
 * @template T
 *
 * @param array<T>|T $value
 * @param T ...$values
 *
 * @return T
 *
 * @pure
 */
function min(mixed $value, mixed ...$values): mixed
{
}

/**
 * @template T
 *
 * @param array<T>|T $value
 * @param T ...$values
 *
 * @return T
 *
 * @pure
 */
function max(mixed $value, mixed ...$values): mixed
{
}

/**
 * @template V
 *
 * @param V $needle
 * @param array<V> $haystack
 *
 * @pure
 */
function in_array(mixed $needle, array $haystack, bool $strict = false): bool
{
}

/**
 * @template K of array-key
 * @template V
 *
 * @param V $needle
 * @param array<K, V> $haystack
 *
 * @return K|false
 *
 * @pure
 */
function array_search(mixed $needle, array $haystack, bool $strict = false): string|int|false
{
}

/**
 * @template T
 *
 * @param T $value
 *
 * @return (
 *   $start_index is 0 ?
 *   ($count is int<1, max> ? non-empty-list<T> : list<T>) :
 *   ($count is int<1, max> ? non-empty-array<int, T> : array<int, T>)
 * )
 *
 * @pure
 */
function array_fill(int $start_index, int $count, mixed $value): array
{
}

/**
 * @template K of array-key
 * @template V
 *
 * @param array<K> $keys
 * @param V $value
 *
 * @return ($keys is non-empty-array<K> ? non-empty-array<K, V> : array<K, V>)
 *
 * @pure
 */
function array_fill_keys(array $keys, mixed $value): array
{
}

/**
 * @template K of array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param V ...$values
 *
 * @param-out ($array is list<V> ? non-empty-list<V> : non-empty-array<K, V>) $array
 *
 * @return int<1, max>
 *
 * @pure
 */
function array_push(array &$array, mixed ...$values): int
{
}

/**
 * @template K of array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @param-out ($array is list<V> ? list<V> : array<K, V>) $array
 *
 * @return ($array is non-empty-array|non-empty-list ? V : V|null)
 */
function array_pop(array &$array): mixed
{
}

/**
 * @template K of array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param-out ($array is list<V> ? list<V> : array<K, V>) $array
 *
 * @return ($array is non-empty-array|non-empty-list ? V : V|null)
 *
 * @pure
 */
function array_shift(array &$array): mixed
{
}

/**
 * @template K of array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @return ($preserve_keys is true ? array<K, V> : list<V>) $array
 *
 * @pure
 */
function array_slice(array $array, int $offset, null|int $length = null, bool $preserve_keys = false): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> ...$arrays
 *
 * @return array<K, V>
 *
 * @pure
 */
function array_merge(array ...$arrays): array
{
}

/**
 * @param (callable(string, null|int): string)|null $callback
 */
function ob_start(callable|null $callback = null, int $chunk_size = 0, int $flags = PHP_OUTPUT_HANDLER_STDFLAGS): bool
{
}

function ob_flush(): bool
{
}

function ob_clean(): bool
{
}

function ob_end_flush(): bool
{
}

function ob_end_clean(): bool
{
}

function ob_get_flush(): string|false
{
}

function ob_get_clean(): string|false
{
}

function ob_get_length(): int|false
{
}

function ob_get_level(): int
{
}

/**
 * @return array{
 *   level: int,
 *   type: int,
 *   flags: int,
 *   name: string,
 *   del: int,
 *   chunk_size: int,
 *   buffer_size: int,
 *   buffer_used: int
 * }
 */
function ob_get_status(bool $full_status = false): array
{
}

function ob_get_contents(): string|false
{
}

function ob_implicit_flush(bool $enable = true): void
{
}

/**
 * @return list<non-empty-string>
 */
function ob_list_handlers(): array
{
}

function syslog(int $priority, string $message): true
{
}

function closelog(): true
{
}

/**
 * @param (callable(): void) $callback
 */
function header_register_callback(callable $callback): bool
{
}

/**
 * @return false|array{
 *  0: int,
 *  1: int,
 *  2: int,
 *  3: string,
 *  bits: int,
 *  channels: int,
 *  mime: string
 * }
 */
function getimagesizefromstring(string $string, &$image_info): array|false
{
}

/**
 * @param resource $stream
 */
function stream_set_chunk_size($stream, int $size): int
{
}

/**
 * @pure
 */
function metaphone(string $string, int $max_phonemes = 0): string
{
}

function array_walk(object|array &$array, callable $callback, mixed $arg = null): true
{
}

function array_walk_recursive(object|array &$array, callable $callback, mixed $arg = null): true
{
}

/**
 * @param array<string, mixed> $array
 *
 * @return int<0, max>
 */
function extract(array &$array, int $flags = EXTR_OVERWRITE, string $prefix = ''): int
{
}

/**
 * @param string|array<string> $var_name
 * @param string|array<string> ...$var_names
 *
 * @return array<string, mixed>
 *
 * @pure
 */
function compact($var_name, ...$var_names): array
{
}

/**
 * @param positive-int|float $step
 *
 * @return (
 *   $start is string
 *   ? (
 *     $end is string
 *     ? ($step is int ? list<string> : list{float})
 *     : ($end is float ? list<float> : ($step is float ? list<float> : list<int>))
 *   )
 *   : (
 *     $start is float
 *     ? list<float>
 *     : ($end is float ? list<float> : ($step is float ? list<float> : list<int>))
 *   )
 * )
 *
 * @pure
 */
function range(string|int|float $start, string|int|float $end, int|float $step = 1): array
{
}

/**
 * @pure
 */
function array_multisort(&$array, $sort_order = SORT_ASC, $sort_flags = SORT_REGULAR, &...$rest): bool
{
}

/**
 * @template K of array-key
 * @template V
 * @template T
 *
 * @param array<K, V> $array
 * @param T ...$values
 *
 * @param-out array<K, V|T> $array
 *
 * @return int<0, max>
 *
 * @pure
 */
function array_unshift(array &$array, mixed ...$values): int
{
}

/**
 * @pure
 */
function array_splice(array &$array, int $offset, null|int $length = null, mixed $replacement = []): array
{
}
