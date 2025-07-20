<?php

/**
 * @var int
 */
const SNMP_OID_OUTPUT_SUFFIX = UNKNOWN;

/**
 * @var int
 */
const SNMP_OID_OUTPUT_MODULE = UNKNOWN;

/**
 * @var int
 */
const SNMP_OID_OUTPUT_FULL = UNKNOWN;

/**
 * @var int
 */
const SNMP_OID_OUTPUT_NUMERIC = UNKNOWN;

/**
 * @var int
 */
const SNMP_OID_OUTPUT_UCD = UNKNOWN;

/**
 * @var int
 */
const SNMP_OID_OUTPUT_NONE = UNKNOWN;

/**
 * @var int
 */
const SNMP_VALUE_LIBRARY = UNKNOWN;

/**
 * @var int
 */
const SNMP_VALUE_PLAIN = UNKNOWN;

/**
 * @var int
 */
const SNMP_VALUE_OBJECT = UNKNOWN;

/**
 * @var int
 */
const SNMP_BIT_STR = UNKNOWN;

/**
 * @var int
 */
const SNMP_OCTET_STR = UNKNOWN;

/**
 * @var int
 */
const SNMP_OPAQUE = UNKNOWN;

/**
 * @var int
 */
const SNMP_NULL = UNKNOWN;

/**
 * @var int
 */
const SNMP_OBJECT_ID = UNKNOWN;

/**
 * @var int
 */
const SNMP_IPADDRESS = UNKNOWN;

/**
 * @var int
 */
const SNMP_COUNTER = UNKNOWN;

/**
 * @var int
 */
const SNMP_UNSIGNED = UNKNOWN;

/**
 * @var int
 */
const SNMP_TIMETICKS = UNKNOWN;

/**
 * @var int
 */
const SNMP_UINTEGER = UNKNOWN;

/**
 * @var int
 */
const SNMP_INTEGER = UNKNOWN;

/**
 * @var int
 */
const SNMP_COUNTER64 = UNKNOWN;

function snmpget(
    string $hostname,
    string $community,
    array|string $object_id,
    int $timeout = -1,
    int $retries = -1,
): mixed {
}

function snmpgetnext(
    string $hostname,
    string $community,
    array|string $object_id,
    int $timeout = -1,
    int $retries = -1,
): mixed {
}

function snmpwalk(
    string $hostname,
    string $community,
    array|string $object_id,
    int $timeout = -1,
    int $retries = -1,
): array|false {
}

function snmprealwalk(
    string $hostname,
    string $community,
    array|string $object_id,
    int $timeout = -1,
    int $retries = -1,
): array|false {
}

/** @alias snmprealwalk */
function snmpwalkoid(
    string $hostname,
    string $community,
    array|string $object_id,
    int $timeout = -1,
    int $retries = -1,
): array|false {
}

function snmpset(
    string $hostname,
    string $community,
    array|string $object_id,
    array|string $type,
    array|string $value,
    int $timeout = -1,
    int $retries = -1,
): bool {
}

function snmp_get_quick_print(): bool
{
}

function snmp_set_quick_print(bool $enable): true
{
}

function snmp_set_enum_print(bool $enable): true
{
}

function snmp_set_oid_output_format(int $format): true
{
}

/** @alias snmp_set_oid_output_format */
function snmp_set_oid_numeric_print(int $format): true
{
}

function snmp2_get(
    string $hostname,
    string $community,
    array|string $object_id,
    int $timeout = -1,
    int $retries = -1,
): mixed {
}

function snmp2_getnext(
    string $hostname,
    string $community,
    array|string $object_id,
    int $timeout = -1,
    int $retries = -1,
): mixed {
}

function snmp2_walk(
    string $hostname,
    string $community,
    array|string $object_id,
    int $timeout = -1,
    int $retries = -1,
): array|false {
}

function snmp2_real_walk(
    string $hostname,
    string $community,
    array|string $object_id,
    int $timeout = -1,
    int $retries = -1,
): array|false {
}

function snmp2_set(
    string $hostname,
    string $community,
    array|string $object_id,
    array|string $type,
    array|string $value,
    int $timeout = -1,
    int $retries = -1,
): bool {
}

function snmp3_get(
    string $hostname,
    string $security_name,
    string $security_level,
    string $auth_protocol,
    string $auth_passphrase,
    string $privacy_protocol,
    string $privacy_passphrase,
    array|string $object_id,
    int $timeout = -1,
    int $retries = -1,
): mixed {
}

function snmp3_getnext(
    string $hostname,
    string $security_name,
    string $security_level,
    string $auth_protocol,
    string $auth_passphrase,
    string $privacy_protocol,
    string $privacy_passphrase,
    array|string $object_id,
    int $timeout = -1,
    int $retries = -1,
): mixed {
}

function snmp3_walk(
    string $hostname,
    string $security_name,
    string $security_level,
    string $auth_protocol,
    string $auth_passphrase,
    string $privacy_protocol,
    string $privacy_passphrase,
    array|string $object_id,
    int $timeout = -1,
    int $retries = -1,
): array|false {
}

function snmp3_real_walk(
    string $hostname,
    string $security_name,
    string $security_level,
    string $auth_protocol,
    string $auth_passphrase,
    string $privacy_protocol,
    string $privacy_passphrase,
    array|string $object_id,
    int $timeout = -1,
    int $retries = -1,
): array|false {
}

function snmp3_set(
    string $hostname,
    string $security_name,
    string $security_level,
    string $auth_protocol,
    string $auth_passphrase,
    string $privacy_protocol,
    string $privacy_passphrase,
    array|string $object_id,
    array|string $type,
    array|string $value,
    int $timeout = -1,
    int $retries = -1,
): bool {
}

function snmp_set_valueretrieval(int $method): true
{
}

function snmp_get_valueretrieval(): int
{
}

function snmp_read_mib(string $filename): bool
{
}

class SNMP
{
    public const int VERSION_1 = UNKNOWN;
    public const int VERSION_2c = UNKNOWN;
    public const int VERSION_2C = UNKNOWN;
    public const int VERSION_3 = UNKNOWN;
    public const int ERRNO_NOERROR = UNKNOWN;
    public const int ERRNO_ANY = UNKNOWN;
    public const int ERRNO_GENERIC = UNKNOWN;
    public const int ERRNO_TIMEOUT = UNKNOWN;
    public const int ERRNO_ERROR_IN_REPLY = UNKNOWN;
    public const int ERRNO_OID_NOT_INCREASING = UNKNOWN;
    public const int ERRNO_OID_PARSING_ERROR = UNKNOWN;
    public const int ERRNO_MULTIPLE_SET_QUERIES = UNKNOWN;

    /** @readonly */
    public array $info;
    public null|int $max_oids;
    public int $valueretrieval;
    public bool $quick_print;
    public bool $enum_print;
    public int $oid_output_format;
    public bool $oid_increasing_check;
    public int $exceptions_enabled;

    public function __construct(
        int $version,
        string $hostname,
        string $community,
        int $timeout = -1,
        int $retries = -1,
    ) {}

    public function close(): bool
    {
    }

    public function setSecurity(
        string $securityLevel,
        string $authProtocol = '',
        string $authPassphrase = '',
        string $privacyProtocol = '',
        string $privacyPassphrase = '',
        string $contextName = '',
        string $contextEngineId = '',
    ): bool {
    }

    public function get(array|string $objectId, bool $preserveKeys = false): mixed
    {
    }

    public function getnext(array|string $objectId): mixed
    {
    }

    public function walk(
        array|string $objectId,
        bool $suffixAsKey = false,
        int $maxRepetitions = -1,
        int $nonRepeaters = -1,
    ): array|false {
    }

    public function set(array|string $objectId, array|string $type, array|string $value): bool
    {
    }

    public function getErrno(): int
    {
    }

    public function getError(): string
    {
    }
}

class SNMPException extends RuntimeException
{
}
