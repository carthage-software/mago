<?php

/**
 * @var string
 */
const DATE_ATOM = "Y-m-d\\TH:i:sP";

/**
 * @var string
 */
const DATE_COOKIE = 'l, d-M-Y H:i:s T';

/**
 * @var string
 */
const DATE_ISO8601 = "Y-m-d\\TH:i:sO";

/**
 * @var string
 */
const DATE_ISO8601_EXPANDED = "X-m-d\\TH:i:sP";

/**
 * @var string
 */
const DATE_RFC822 = 'D, d M y H:i:s O';

/**
 * @var string
 */
const DATE_RFC850 = 'l, d-M-y H:i:s T';

/**
 * @var string
 */
const DATE_RFC1036 = 'D, d M y H:i:s O';

/**
 * @var string
 */
const DATE_RFC1123 = 'D, d M Y H:i:s O';

/**
 * @var string
 */
const DATE_RFC7231 = "D, d M Y H:i:s \\G\\M\\T";

/**
 * @var string
 */
const DATE_RFC2822 = 'D, d M Y H:i:s O';

/**
 * @var string
 */
const DATE_RFC3339 = "Y-m-d\\TH:i:sP";

/**
 * @var string
 */
const DATE_RFC3339_EXTENDED = "Y-m-d\\TH:i:s.vP";

/** @var string */
const DATE_RSS = DATE_RFC1123;

/** @var string */
const DATE_W3C = DATE_RFC3339;

/**
 * @var int
 * @deprecated
 */
const SUNFUNCS_RET_TIMESTAMP = UNKNOWN;

/**
 * @var int
 * @deprecated
 */
const SUNFUNCS_RET_STRING = UNKNOWN;

/**
 * @var int
 * @deprecated
 */
const SUNFUNCS_RET_DOUBLE = UNKNOWN;

function strtotime(string $datetime, null|int $baseTimestamp = null): int|false
{
}

function date(string $format, null|int $timestamp = null): string
{
}

function idate(string $format, null|int $timestamp = null): int|false
{
}

function gmdate(string $format, null|int $timestamp = null): string
{
}

function mktime(
    int $hour,
    null|int $minute = null,
    null|int $second = null,
    null|int $month = null,
    null|int $day = null,
    null|int $year = null,
): int|false {
}

function gmmktime(
    int $hour,
    null|int $minute = null,
    null|int $second = null,
    null|int $month = null,
    null|int $day = null,
    null|int $year = null,
): int|false {
}

function checkdate(int $month, int $day, int $year): bool
{
}

/**
 * @deprecated
 */
function strftime(string $format, null|int $timestamp = null): string|false
{
}

/**
 * @deprecated
 */
function gmstrftime(string $format, null|int $timestamp = null): string|false
{
}

function time(): int
{
}

/**
 * @return array<int|string, int>
 */
function localtime(null|int $timestamp = null, bool $associative = false): array
{
}

/**
 * @return array{
 *   seconds: int,
 *   minutes: int,
 *   hours: int,
 *   mday: int,
 *   wday: int,
 *   mon: int,
 *   year: int,
 *   yday: int,
 *   weekday: int,
 *   month: string,
 *   0: int
 * }
 */
function getdate(null|int $timestamp): array
{
}

function date_create(string $datetime = 'now', null|DateTimeZone $timezone = null): DateTime|false
{
}

function date_create_immutable(string $datetime = 'now', null|DateTimeZone $timezone = null): DateTimeImmutable|false
{
}

function date_create_from_format(string $format, string $datetime, null|DateTimeZone $timezone = null): DateTime|false
{
}

function date_create_immutable_from_format(
    string $format,
    string $datetime,
    null|DateTimeZone $timezone = null,
): DateTimeImmutable|false {
}

/**
 * @return array<string, mixed>
 */
function date_parse(string $datetime): array
{
}

/**
 * @return array<string, mixed>
 */
function date_parse_from_format(string $format, string $datetime): array
{
}

/**
 * @return array<string, int|array>|false
 */
function date_get_last_errors(): array|false
{
}

function date_format(DateTimeInterface $object, string $format): string
{
}

function date_modify(DateTime $object, string $modifier): DateTime|false
{
}

function date_add(DateTime $object, DateInterval $interval): DateTime
{
}

function date_sub(DateTime $object, DateInterval $interval): DateTime
{
}

function date_timezone_get(DateTimeInterface $object): DateTimeZone|false
{
}

function date_timezone_set(DateTime $object, DateTimeZone $timezone): DateTime
{
}

function date_offset_get(DateTimeInterface $object): int
{
}

function date_diff(DateTimeInterface $baseObject, DateTimeInterface $targetObject, bool $absolute = false): DateInterval
{
}

function date_time_set(DateTime $object, int $hour, int $minute, int $second = 0, int $microsecond = 0): DateTime
{
}

function date_date_set(DateTime $object, int $year, int $month, int $day): DateTime
{
}

function date_isodate_set(DateTime $object, int $year, int $week, int $dayOfWeek = 1): DateTime
{
}

function date_timestamp_set(DateTime $object, int $timestamp): DateTime
{
}

function date_timestamp_get(DateTimeInterface $object): int
{
}

function timezone_open(string $timezone): DateTimeZone|false
{
}

function timezone_name_get(DateTimeZone $object): string
{
}

function timezone_name_from_abbr(string $abbr, int $utcOffset = -1, int $isDST = -1): string|false
{
}

function timezone_offset_get(DateTimeZone $object, DateTimeInterface $datetime): int
{
}

/**
 * @return array<int, array>|false
 */
function timezone_transitions_get(
    DateTimeZone $object,
    int $timestampBegin = PHP_INT_MIN,
    int $timestampEnd = 2147483647,
): array|false {
}

/**
 * @return array<string, float|string>|false
 */
function timezone_location_get(DateTimeZone $object): array|false
{
}

/**
 * @return array<int, string>
 */
function timezone_identifiers_list(int $timezoneGroup = DateTimeZone::ALL, null|string $countryCode = null): array
{
}

/**
 * @return array<string, array>
 */
function timezone_abbreviations_list(): array
{
}

function timezone_version_get(): string
{
}

function date_interval_create_from_date_string(string $datetime): DateInterval|false
{
}

function date_interval_format(DateInterval $object, string $format): string
{
}

function date_default_timezone_set(string $timezoneId): bool
{
}

function date_default_timezone_get(): string
{
}

/**
 * @deprecated
 */
function date_sunrise(
    int $timestamp,
    int $returnFormat = SUNFUNCS_RET_STRING,
    null|float $latitude = null,
    null|float $longitude = null,
    null|float $zenith = null,
    null|float $utcOffset = null,
): string|int|float|false {
}

/**
 * @deprecated
 */
function date_sunset(
    int $timestamp,
    int $returnFormat = SUNFUNCS_RET_STRING,
    null|float $latitude = null,
    null|float $longitude = null,
    null|float $zenith = null,
    null|float $utcOffset = null,
): string|int|float|false {
}

/**
 * @return array{
 *   sunrise: int|bool,
 *   sunset: int|bool,
 *   transit: int|bool,
 *   civil_twilight_begin: int|bool,
 *   civil_twilight_end: int|bool,
 *   nautical_twilight_begin: int|bool,
 *   nautical_twilight_end: int|bool,
 *   astronomical_twilight_begin: int|bool,
 *   astronomical_twilight_end: int|bool,
 * }
 */
function date_sun_info(int $timestamp, float $latitude, float $longitude): array
{
}

interface DateTimeInterface
{
    public const string ATOM = DATE_ATOM;

    public const string COOKIE = DATE_COOKIE;

    public const string ISO8601 = DATE_ISO8601;

    public const string ISO8601_EXPANDED = DATE_ISO8601_EXPANDED;

    public const string RFC822 = DATE_RFC822;

    public const string RFC850 = DATE_RFC850;

    public const string RFC1036 = DATE_RFC1036;

    public const string RFC1123 = DATE_RFC1123;

    public const string RFC7231 = DATE_RFC7231;

    public const string RFC2822 = DATE_RFC2822;

    public const string RFC3339 = DATE_RFC3339;

    public const string RFC3339_EXTENDED = DATE_RFC3339_EXTENDED;

    public const string RSS = DATE_RSS;

    public const string W3C = DATE_W3C;

    public function format(string $format): string;

    public function getTimezone(): DateTimeZone|false;

    public function getOffset(): int;

    public function getTimestamp(): int;

    public function getMicrosecond(): int;

    public function diff(DateTimeInterface $targetObject, bool $absolute = false): DateInterval;

    public function __wakeup(): void;

    public function __serialize(): array;

    public function __unserialize(array $data): void;
}

class DateTime implements DateTimeInterface
{
    public function __construct(string $datetime = 'now', null|DateTimeZone $timezone = null) {}

    public function __serialize(): array
    {
    }

    public function __unserialize(array $data): void
    {
    }

    public function __wakeup(): void
    {
    }

    public static function __set_state(array $array): DateTime
    {
    }

    public static function createFromImmutable(DateTimeImmutable $object): static
    {
    }

    /** @return static */
    public static function createFromInterface(DateTimeInterface $object): DateTime
    {
    } // TODO return type should be static

    public static function createFromFormat(
        string $format,
        string $datetime,
        null|DateTimeZone $timezone = null,
    ): DateTime|false {
    }

    public static function createFromTimestamp(int|float $timestamp): static
    {
    }

    /**
     * @return array<string, int|array>|false
     */
    public static function getLastErrors(): array|false
    {
    }

    public function format(string $format): string
    {
    }

    public function modify(string $modifier): DateTime
    {
    }

    public function add(DateInterval $interval): DateTime
    {
    }

    public function sub(DateInterval $interval): DateTime
    {
    }

    public function getTimezone(): DateTimeZone|false
    {
    }

    public function setTimezone(DateTimeZone $timezone): DateTime
    {
    }

    public function getOffset(): int
    {
    }

    public function getMicrosecond(): int
    {
    }

    public function setTime(int $hour, int $minute, int $second = 0, int $microsecond = 0): DateTime
    {
    }

    public function setDate(int $year, int $month, int $day): DateTime
    {
    }

    public function setISODate(int $year, int $week, int $dayOfWeek = 1): DateTime
    {
    }

    public function setTimestamp(int $timestamp): DateTime
    {
    }

    public function setMicrosecond(int $microsecond): static
    {
    }

    public function getTimestamp(): int
    {
    }

    public function diff(DateTimeInterface $targetObject, bool $absolute = false): DateInterval
    {
    }
}

class DateTimeImmutable implements DateTimeInterface
{
    public function __construct(string $datetime = 'now', null|DateTimeZone $timezone = null) {}

    public function __serialize(): array
    {
    }

    public function __unserialize(array $data): void
    {
    }

    public function __wakeup(): void
    {
    }

    public static function __set_state(array $array): DateTimeImmutable
    {
    }

    public static function createFromFormat(
        string $format,
        string $datetime,
        null|DateTimeZone $timezone = null,
    ): DateTimeImmutable|false {
    }

    public static function createFromTimestamp(int|float $timestamp): static
    {
    }

    /**
     * @return array<string, int|array>|false
     */
    public static function getLastErrors(): array|false
    {
    }

    public function format(string $format): string
    {
    }

    public function getTimezone(): DateTimeZone|false
    {
    }

    public function getOffset(): int
    {
    }

    public function getTimestamp(): int
    {
    }

    public function getMicrosecond(): int
    {
    }

    public function diff(DateTimeInterface $targetObject, bool $absolute = false): DateInterval
    {
    }

    #[NoDiscard(message: 'as DateTimeImmutable::modify() does not modify the object itself')]
    public function modify(string $modifier): DateTimeImmutable
    {
    }

    #[NoDiscard(message: 'as DateTimeImmutable::add() does not modify the object itself')]
    public function add(DateInterval $interval): DateTimeImmutable
    {
    }

    #[NoDiscard(message: 'as DateTimeImmutable::sub() does not modify the object itself')]
    public function sub(DateInterval $interval): DateTimeImmutable
    {
    }

    #[NoDiscard(message: 'as DateTimeImmutable::setTimezone() does not modify the object itself')]
    public function setTimezone(DateTimeZone $timezone): DateTimeImmutable
    {
    }

    #[NoDiscard(message: 'as DateTimeImmutable::setTime() does not modify the object itself')]
    public function setTime(int $hour, int $minute, int $second = 0, int $microsecond = 0): DateTimeImmutable
    {
    }

    #[NoDiscard(message: 'as DateTimeImmutable::setDate() does not modify the object itself')]
    public function setDate(int $year, int $month, int $day): DateTimeImmutable
    {
    }

    #[NoDiscard(message: 'as DateTimeImmutable::setISODate() does not modify the object itself')]
    public function setISODate(int $year, int $week, int $dayOfWeek = 1): DateTimeImmutable
    {
    }

    #[NoDiscard(message: 'as DateTimeImmutable::setTimestamp() does not modify the object itself')]
    public function setTimestamp(int $timestamp): DateTimeImmutable
    {
    }

    #[NoDiscard(message: 'as DateTimeImmutable::setMicrosecond() does not modify the object itself')]
    public function setMicrosecond(int $microsecond): static
    {
    }

    public static function createFromMutable(DateTime $object): static
    {
    }

    /**
     * @return static
     */
    public static function createFromInterface(DateTimeInterface $object): DateTimeImmutable
    {
    }
}

class DateTimeZone
{
    public const int AFRICA = UNKNOWN;
    public const int AMERICA = UNKNOWN;
    public const int ANTARCTICA = UNKNOWN;
    public const int ARCTIC = UNKNOWN;
    public const int ASIA = UNKNOWN;
    public const int ATLANTIC = UNKNOWN;
    public const int AUSTRALIA = UNKNOWN;
    public const int EUROPE = UNKNOWN;
    public const int INDIAN = UNKNOWN;
    public const int PACIFIC = UNKNOWN;
    public const int UTC = UNKNOWN;
    public const int ALL = UNKNOWN;
    public const int ALL_WITH_BC = UNKNOWN;
    public const int PER_COUNTRY = UNKNOWN;

    public function __construct(string $timezone) {}

    public function getName(): string
    {
    }

    public function getOffset(DateTimeInterface $datetime): int
    {
    }

    /**
     * @return array<int, array>|false
     */
    public function getTransitions(int $timestampBegin = PHP_INT_MIN, int $timestampEnd = 2147483647): array|false
    {
    }

    /**
     * @return array<string, float|string>|false
     */
    public function getLocation(): array|false
    {
    }

    /**
     * @return array<string, array>
     */
    public static function listAbbreviations(): array
    {
    }

    /**
     * @return array<int, string>
     */
    public static function listIdentifiers(
        int $timezoneGroup = DateTimeZone::ALL,
        null|string $countryCode = null,
    ): array {
    }

    public function __serialize(): array
    {
    }

    public function __unserialize(array $data): void
    {
    }

    public function __wakeup(): void
    {
    }

    public static function __set_state(array $array): DateTimeZone
    {
    }
}

class DateInterval
{
    public readonly int $y;
    public readonly int $m;
    public readonly int $d;
    public readonly int $h;
    public readonly int $i;
    public readonly int $s;
    public readonly float $f;
    /** @var int<0, 1> */
    public readonly int $invert;
    public readonly int|false $days;
    public readonly bool $from_string;
    public readonly string $date_string;

    public function __construct(string $duration) {}

    public static function createFromDateString(string $datetime): DateInterval
    {
    }

    public function format(string $format): string
    {
    }

    public function __serialize(): array;

    public function __unserialize(array $data): void;

    public function __wakeup(): void
    {
    }

    public static function __set_state(array $array): DateInterval
    {
    }
}

class DatePeriod implements IteratorAggregate
{
    public const int EXCLUDE_START_DATE = UNKNOWN;
    public const int INCLUDE_END_DATE = UNKNOWN;

    /**
     * @readonly
     */
    public null|DateTimeInterface $start;

    /**
     * @readonly
     */
    public null|DateTimeInterface $current;

    /**
     * @readonly
     */
    public null|DateTimeInterface $end;

    /**
     * @readonly
     */
    public null|DateInterval $interval;

    /**
     * @readonly
     */
    public int $recurrences;

    /**
     * @readonly
     */
    public bool $include_start_date;

    /**
     * @readonly
     */
    public bool $include_end_date;

    public static function createFromISO8601String(string $specification, int $options = 0): static
    {
    }

    /**
     * @param DateTimeInterface|string $start
     * @param DateInterval|int $interval
     * @param DateTimeInterface|int $end
     * @param int $options
     */
    public function __construct($start, $interval = UNKNOWN, $end = UNKNOWN, $options = UNKNOWN) {}

    public function getStartDate(): DateTimeInterface
    {
    }

    public function getEndDate(): null|DateTimeInterface
    {
    }

    public function getDateInterval(): DateInterval
    {
    }

    public function getRecurrences(): null|int
    {
    }

    public function __serialize(): array;

    public function __unserialize(array $data): void;

    public function __wakeup(): void
    {
    }

    public static function __set_state(array $array): DatePeriod
    {
    }

    public function getIterator(): Iterator
    {
    }
}

class DateError extends Error
{
}

class DateObjectError extends DateError
{
}

class DateRangeError extends DateError
{
}

class DateException extends Exception
{
}

class DateInvalidTimeZoneException extends DateException
{
}

class DateInvalidOperationException extends DateException
{
}

class DateMalformedStringException extends DateException
{
}

class DateMalformedIntervalStringException extends DateException
{
}

class DateMalformedPeriodStringException extends DateException
{
}
