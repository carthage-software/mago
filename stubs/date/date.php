<?php

use JetBrains\PhpStorm\ArrayShape;
use JetBrains\PhpStorm\Deprecated;
use JetBrains\PhpStorm\Internal\LanguageLevelTypeAware;
use JetBrains\PhpStorm\Internal\PhpStormStubsElementAvailable;
use JetBrains\PhpStorm\Pure;

function strtotime(string $datetime, null|int $baseTimestamp): int|false
{
}

function date(string $format, null|int $timestamp): string
{
}

function idate(string $format, null|int $timestamp): int|false
{
}

function gmdate(string $format, null|int $timestamp): string
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

/**
 * @pure
 */
function checkdate(int $month, int $day, int $year): bool
{
}

/**
 * @deprecated
 */
function strftime(string $format, null|int $timestamp): string|false
{
}

/**
 * @deprecated
 */
function gmstrftime(string $format, null|int $timestamp): string|false
{
}

/**
 * Return current Unix timestamp
 * @link https://php.net/manual/en/function.time.php
 * @return int <p>Returns the current time measured in the number of seconds since the Unix Epoch (January 1 1970 00:00:00 GMT).</p>
 */
function time(): int
{
}

/**
 * Get the local time
 * @link https://php.net/manual/en/function.localtime.php
 * @param int|null $timestamp [optional]
 * @param bool $associative [optional] <p>
 * If set to false or not supplied then the array is returned as a regular,
 * numerically indexed array. If the argument is set to true then
 * localtime returns an associative array containing
 * all the different elements of the structure returned by the C
 * function call to localtime. The names of the different keys of
 * the associative array are as follows:
 * </p>
 * "tm_sec" - seconds
 * @return array
 */
#[Pure(true)]
#[ArrayShape([
    'tm_sec' => 'int',
    'tm_min' => 'int',
    'tm_hour' => 'int',
    'tm_mday' => 'int',
    'tm_mon' => 'int',
    'tm_year' => 'int',
    'tm_wday' => 'int',
    'tm_yday' => 'int',
    'tm_isdst' => 'int',
])]
function localtime(null|int $timestamp, bool $associative = false): array
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

function date_create(string $datetime = 'now', null|DateTimeZone $timezone): DateTime|false
{
}

function date_create_immutable(string $datetime = 'now', null|DateTimeZone $timezone): DateTimeImmutable|false
{
}

function date_create_immutable_from_format(
    string $format,
    string $datetime,
    null|DateTimeZone $timezone,
): DateTimeImmutable|false {
}

/**
 * Alias:
 * {@see DateTime::createFromFormat}
 * @link https://php.net/manual/en/function.date-create-from-format.php
 * @param string $format Format accepted by  <a href="https://secure.php.net/manual/en/function.date.php">date()</a>.
 * <p>If format does not contain the character ! then portions of the generated time which are not specified in format will be set to the current system time.</p>
 * <p>If format contains the character !, then portions of the generated time not provided in format, as well as values to the left-hand side of the !, will be set to corresponding values from the Unix epoch.</p>
 * <p>The Unix epoch is 1970-01-01 00:00:00 UTC.</p>
 * @param string $datetime String representing the time.
 * @param DateTimeZone|null $timezone [optional] A DateTimeZone object representing the desired time zone.
 * @return DateTime|false <p> Returns a new
 * {@see DateTime} instance or <b>FALSE</b> on failure.</p>
 */
#[Pure(true)]
function date_create_from_format(string $format, string $datetime, null|DateTimeZone $timezone): DateTime|false
{
}

/**
 * Returns associative array with detailed info about given date
 * @link https://php.net/manual/en/function.date-parse.php
 * @param string $datetime <p>
 * Date in format accepted by strtotime.
 * </p>
 * @return array|false array with information about the parsed date
 * on success or false on failure.
 */
#[Pure(true)]
#[LanguageLevelTypeAware(['8.0' => 'array'], default: 'array|false')]
#[ArrayShape([
    'year' => 'int',
    'month' => 'int',
    'day' => 'int',
    'hour' => 'int',
    'minute' => 'int',
    'second' => 'int',
    'fraction' => 'double',
    'is_localtime' => 'bool',
    'zone_type' => 'int',
    'zone' => 'int',
    'is_dst' => 'bool',
    'tz_abbr' => 'string',
    'tz_id' => 'string',
    'relative' => 'array',
    'warning_count' => 'int',
    'warnings' => 'array',
    'error_count' => 'int',
    'errors' => 'array',
])]
function date_parse(string $datetime): false|array
{
}

/**
 * Get info about given date formatted according to the specified format
 * @link https://php.net/manual/en/function.date-parse-from-format.php
 * @param string $format <p>
 * Format accepted by date with some extras.
 * </p>
 * @param string $datetime <p>
 * String representing the date.
 * </p>
 * @return array associative array with detailed info about given date.
 */
#[Pure(true)]
#[ArrayShape([
    'year' => 'int',
    'month' => 'int',
    'day' => 'int',
    'hour' => 'int',
    'minute' => 'int',
    'second' => 'int',
    'fraction' => 'double',
    'is_localtime' => 'bool',
    'zone_type' => 'int',
    'zone' => 'int',
    'is_dst' => 'bool',
    'tz_abbr' => 'string',
    'tz_id' => 'string',
    'relative' => 'array',
    'warning_count' => 'int',
    'warnings' => 'array',
    'error_count' => 'int',
    'errors' => 'array',
])]
function date_parse_from_format(string $format, string $datetime): array
{
}

/**
 * Returns the warnings and errors
 * Alias:
 * {@see DateTime::getLastErrors}
 * @link https://php.net/manual/en/function.date-get-last-errors.php
 * @return array|false <p>Returns array containing info about warnings and errors.</p>
 */
#[ArrayShape(['warning_count' => 'int', 'warnings' => 'string[]', 'error_count' => 'int', 'errors' => 'string[]'])]
#[Pure(true)]
function date_get_last_errors(): array|false
{
}

/**
 * Alias:
 * {@see DateTime::format}
 * @link https://php.net/manual/en/function.date-format.php
 * @param DateTimeInterface $object
 * @param string $format
 * @return string|false formatted date string on success or <b>FALSE</b> on failure.
 */
#[Pure(true)]
#[LanguageLevelTypeAware(['8.0' => 'string'], default: 'string|false')]
function date_format(DateTimeInterface $object, string $format)
{
}

/**
 * Alter the timestamp of a DateTime object by incrementing or decrementing
 * in a format accepted by strtotime().
 * Alias:
 * {@see DateTime::modify}
 * @link https://php.net/manual/en/function.date-modify.php
 * @param DateTime $object A DateTime object returned by date_create(). The function modifies this object.
 * @param string $modifier A date/time string. Valid formats are explained in {@link https://secure.php.net/manual/en/datetime.formats.php Date and Time Formats}.
 * @return DateTime|false Returns the DateTime object for method chaining or <b>FALSE</b> on failure.
 */
function date_modify(DateTime $object, string $modifier): DateTime|false
{
}

/**
 * Alias:
 * {@see DateTime::add}
 * @link https://php.net/manual/en/function.date-add.php
 * @param DateTime $object <p>Procedural style only: A
 * {@see DateTime} object returned by
 * {@see date_create()}. The function modifies this object.</p>
 * @param DateInterval $interval <p>A
 * {@see DateInterval} object</p>
 * @return DateTime|false <p>Returns the
 * {@see DateTime} object for method chaining or <b>FALSE</b> on failure.</p>
 */
#[LanguageLevelTypeAware(['8.0' => 'DateTime'], default: 'DateTime|false')]
function date_add(DateTime $object, DateInterval $interval)
{
}

/**
 * Subtracts an amount of days, months, years, hours, minutes and seconds from a datetime object
 * Alias:
 * {@see DateTime::sub}
 * @link https://php.net/manual/en/function.date-sub.php
 * @param DateTime $object Procedural style only: A
 * {@see DateTime} object returned by
 * {@see date_create()}. The function modifies this object.
 * @param DateInterval $interval <p>A
 * {@see DateInterval} object</p>
 * @return DateTime|false <p>Returns the
 * {@see DateTime} object for method chaining or <b>FALSE</b> on failure.</p>
 */
#[LanguageLevelTypeAware(['8.0' => 'DateTime'], default: 'DateTime|false')]
function date_sub(DateTime $object, DateInterval $interval)
{
}

/**
 * Alias:
 * {@see DateTime::getTimezone}
 * @link https://php.net/manual/en/function.date-timezone-get.php
 * @param DateTimeInterface $object <p>Procedural style only: A
 * {@see DateTime} object
 * returned by
 * {@see date_create()}</p>
 * @return DateTimeZone|false
 * <p>
 * Returns a
 * {@see DateTimeZone} object on success
 * or <b>FALSE</b> on failure.
 * </p>
 */
#[Pure(true)]
function date_timezone_get(DateTimeInterface $object): DateTimeZone|false
{
}

/**
 * Sets the time zone for the datetime object
 * Alias:
 * {@see DateTime::setTimezone}
 * @link https://php.net/manual/en/function.date-timezone-set.php
 * @param DateTime|DateTimeInterface $object <p>A
 * {@see DateTime} object returned by
 * {@see date_create()}. The function modifies this object.</p>
 * @param DateTimeZone $timezone <p>A
 * {@see DateTimeZone} object representing the desired time zone.</p>
 * @return DateTime|false <p>Returns the
 * {@see DateTime} object for method chaining or <b>FALSE</b> on failure.</p>
 */
#[LanguageLevelTypeAware(['8.0' => 'DateTime'], default: 'DateTime|false')]
function date_timezone_set(
    #[LanguageLevelTypeAware(['8.0' => 'DateTime'], default: 'DateTimeInterface')]  $object,
    DateTimeZone $timezone,
) {
}

/**
 * Alias:
 * {@see DateTime::getOffset}
 * @link https://php.net/manual/en/function.date-offset-get.php
 * @param DateTimeInterface $object <p>Procedural style only: A {@see DateTime} object
 * returned by {@see date_create()}</p>
 * @return int|false <p>Returns the timezone offset in seconds from UTC on success or <b>FALSE</b> on failure.</p>
 */
#[Pure(true)]
#[LanguageLevelTypeAware(['8.0' => 'int'], default: 'int|false')]
function date_offset_get(DateTimeInterface $object)
{
}

/**
 * Returns the difference between two datetime objects
 * Alias:
 * {@see DateTime::diff}
 * @link https://php.net/manual/en/function.date-diff.php
 * @param DateTimeInterface $baseObject
 * @param DateTimeInterface $targetObject The date to compare to
 * @param bool $absolute [optional] Whether to return absolute difference.
 * @return DateInterval|false The DateInterval object representing the difference between the two dates or FALSE on failure.
 */
#[Pure(true)]
#[LanguageLevelTypeAware(['8.0' => 'DateInterval'], default: 'DateInterval|false')]
function date_diff(DateTimeInterface $baseObject, DateTimeInterface $targetObject, bool $absolute = false)
{
}

/**
 * Alias:
 * {@see DateTime::setTime}
 * @link https://php.net/manual/en/function.date-time-set.php
 * @param DateTime $object
 * @param int $hour
 * @param int $minute
 * @param int $second [optional]
 * @param int $microsecond [optional]
 * @return DateTime <p>Returns the
 * {@see DateTime} object for method chaining or <b>FALSE</b> on failure.</p>
 */
function date_time_set(
    DateTime $object,
    int $hour,
    int $minute,
    int $second = 0,
    #[PhpStormStubsElementAvailable(from: '7.1')] int $microsecond = 0,
): DateTime {
}

/**
 * Alias:
 * {@see DateTime::setDate}
 * @link https://php.net/manual/en/function.date-date-set.php
 * @param DateTime $object <p>Procedural style only: A {@see DateTime} object
 * returned by {@see date_create()}.
 * The function modifies this object.</p>
 * @param int $year <p>Year of the date.</p>
 * @param int $month <p>Month of the date.</p>
 * @param int $day <p>Day of the date.</p>
 * @return DateTime|false
 * <p>
 * Returns the
 * {@see DateTime} object for method chaining or <b>FALSE</b> on failure.
 * </p>
 */
#[LanguageLevelTypeAware(['8.0' => 'DateTime'], default: 'DateTime|false')]
function date_date_set(DateTime $object, int $year, int $month, int $day): DateTime|false
{
}

/**
 * Alias:
 * {@see DateTime::setISODate}
 * @link https://php.net/manual/en/function.date-isodate-set.php
 * @param DateTime $object
 * @param int $year <p>Year of the date</p>
 * @param int $week <p>Week of the date.</p>
 * @param int $dayOfWeek [optional] <p>Offset from the first day of the week.</p>
 * @return DateTime|false <p>
 * Returns the {@see DateTime} object for method chaining or <strong><code>FALSE</code></strong> on failure.
 * </p>
 */
#[LanguageLevelTypeAware(['8.0' => 'DateTime'], default: 'DateTime|false')]
function date_isodate_set(DateTime $object, int $year, int $week, int $dayOfWeek = 1)
{
}

/**
 * Sets the date and time based on an unix timestamp
 * Alias:
 * {@see DateTime::setTimestamp}
 * @link https://php.net/manual/en/function.date-timestamp-set.php
 * @param DateTime $object <p>Procedural style only: A
 * {@see DateTime} object returned by
 * {@see date_create()}. The function modifies this object.</p>
 * @param int $timestamp <p>Unix timestamp representing the date.</p>
 * @return DateTime|false
 * {@see DateTime} object for call chaining or <b>FALSE</b> on failure
 */
#[LanguageLevelTypeAware(['8.0' => 'DateTime'], default: 'DateTime|false')]
function date_timestamp_set(DateTime $object, int $timestamp): DateTime|false
{
}

/**
 * Gets the unix timestamp
 * Alias:
 * {@see DateTime::getTimestamp}
 * @link https://php.net/manual/en/function.date-timestamp-get.php
 * @param DateTimeInterface $object
 * @return int <p>Returns the Unix timestamp representing the date.</p>
 */
#[Pure(true)]
function date_timestamp_get(DateTimeInterface $object): int
{
}

/**
 * Returns new DateTimeZone object
 * @link https://php.net/manual/en/function.timezone-open.php
 * @param string $timezone <p>
 * Time zone identifier as full name (e.g. Europe/Prague) or abbreviation
 * (e.g. CET).
 * </p>
 * @return DateTimeZone|false DateTimeZone object on success or false on failure.
 */
#[Pure(true)]
function timezone_open(string $timezone): DateTimeZone|false
{
}

/**
 * Alias:
 * {@see DateTimeZone::getName}
 * @link https://php.net/manual/en/function.timezone-name-get.php
 * @param DateTimeZone $object <p>The
 * {@see DateTimeZone} for which to get a name.</p>
 * @return string One of the timezone names in the list of timezones.
 */
#[Pure]
function timezone_name_get(DateTimeZone $object): string
{
}

/**
 * Returns the timezone name from abbreviation
 * @link https://php.net/manual/en/function.timezone-name-from-abbr.php
 * @param string $abbr <p>
 * Time zone abbreviation.
 * </p>
 * @param int $utcOffset [optional] <p>
 * Offset from GMT in seconds. Defaults to -1 which means that first found
 * time zone corresponding to abbr is returned.
 * Otherwise exact offset is searched and only if not found then the first
 * time zone with any offset is returned.
 * </p>
 * @param int $isDST [optional] <p>
 * Daylight saving time indicator. If abbr doesn't
 * exist then the time zone is searched solely by
 * offset and isdst.
 * </p>
 * @return string|false time zone name on success or false on failure.
 * @since 5.1.3
 */
#[Pure(true)]
function timezone_name_from_abbr(string $abbr, int $utcOffset = -1, int $isDST = -1): string|false
{
}

/**
 * Alias:
 * {@link DateTimeZone::getOffset}
 * @link https://php.net/manual/en/function.timezone-offset-get.php
 * @param DateTimeZone $object <p>Procedural style only: A
 * {@see DateTimeZone} object
 * returned by
 * {@see timezone_open()}</p>
 * @param DateTimeInterface $datetime <p>DateTime that contains the date/time to compute the offset from.</p>
 * @return int|false <p>Returns time zone offset in seconds on success or <b>FALSE</b> on failure.</p>
 */
#[Pure(true)]
#[LanguageLevelTypeAware(['8.0' => 'int'], default: 'int|false')]
function timezone_offset_get(DateTimeZone $object, DateTimeInterface $datetime)
{
}

/**
 * Returns all transitions for the timezone
 * Alias:
 * {@see DateTimeZone::getTransitions}
 * @link https://php.net/manual/en/function.timezone-transitions-get.php
 * @param DateTimeZone $object <p>Procedural style only: A
 * {@see DateTimeZone} object returned by
 * {@see timezone_open()}</p>
 * @param int $timestampBegin [optional] <p>Begin timestamp</p>
 * @param int $timestampEnd [optional] <p>End timestamp</p>
 * @return array|false <p>Returns numerically indexed array containing associative array with all transitions on success or FALSE on failure.</p>
 */
#[Pure(true)]
function timezone_transitions_get(
    DateTimeZone $object,
    int $timestampBegin = PHP_INT_MIN,
    int $timestampEnd = PHP_INT_MAX,
): array|false {
}

/**
 * Alias:
 * {@see DateTimeZone::getLocation}
 * @link https://php.net/manual/en/function.timezone-location-get.php
 * @param DateTimeZone $object <p>Procedural style only: A {@see DateTimeZone} object returned by {@see timezone_open()}</p>
 * @return array|false <p>Array containing location information about timezone.</p>
 */
#[Pure(true)]
#[ArrayShape([
    'country_code' => 'string',
    'latitude' => 'double',
    'longitude' => 'double',
    'comments' => 'string',
])]
function timezone_location_get(DateTimeZone $object): array|false
{
}

/**
 * Returns a numerically indexed array containing all defined timezone identifiers
 * Alias:
 * {@see DateTimeZone::listIdentifiers()}
 * @link https://php.net/manual/en/function.timezone-identifiers-list.php
 * @param int $timezoneGroup [optional] One of DateTimeZone class constants.
 * @param string|null $countryCode [optional] A two-letter ISO 3166-1 compatible country code.
 * Note: This option is only used when $timezoneGroup is set to DateTimeZone::PER_COUNTRY.
 * @return array|false Returns array on success or FALSE on failure.
 */
#[Pure(true)]
#[LanguageLevelTypeAware(['8.0' => 'array'], default: 'array|false')]
function timezone_identifiers_list(int $timezoneGroup = DateTimeZone::ALL, null|string $countryCode)
{
}

/**
 * Returns associative array containing dst, offset and the timezone name
 * Alias:
 * {@see DateTimeZone::listAbbreviations}
 * @link https://php.net/manual/en/function.timezone-abbreviations-list.php
 * @return array<string, list<array{dst: bool, offset: int, timezone_id: string|null}>>|false Array on success or <b>FALSE</b> on failure.
 */
#[Pure]
#[LanguageLevelTypeAware(['8.0' => 'array'], default: 'array|false')]
function timezone_abbreviations_list()
{
}

/**
 * Gets the version of the timezonedb
 * @link https://php.net/manual/en/function.timezone-version-get.php
 * @return string a string.
 */
#[Pure]
function timezone_version_get(): string
{
}

/**
 * Alias:
 * {@see DateInterval::createFromDateString}
 * @link https://php.net/manual/en/function.date-interval-create-from-date-string.php
 * @param string $datetime <p>A date with relative parts. Specifically, the relative formats supported by the parser used for
 * {@see strtotime()} and
 * {@see DateTime} will be used to construct the
 * {@see DateInterval}.</p>
 * @return DateInterval|false
 * <p>Returns a new DateInterval instance.</p>
 */
#[Pure(true)]
function date_interval_create_from_date_string(string $datetime): DateInterval|false
{
}

/**
 * Alias:
 * {@see DateInterval::format}
 * @link https://php.net/manual/en/function.date-interval-format.php
 * @param DateInterval $object
 * @param string $format
 * @return string
 */
#[Pure(true)]
function date_interval_format(DateInterval $object, string $format): string
{
}

/**
 * Sets the default timezone used by all date/time functions in a script
 * @link https://php.net/manual/en/function.date-default-timezone-set.php
 * @param string $timezoneId <p>
 * The timezone identifier, like UTC or
 * Europe/Lisbon. The list of valid identifiers is
 * available in the .
 * </p>
 * @return bool This function returns false if the
 * timezone_identifier isn't valid, or true
 * otherwise.
 */
function date_default_timezone_set(string $timezoneId): bool
{
}

/**
 * Gets the default timezone used by all date/time functions in a script
 * @link https://php.net/manual/en/function.date-default-timezone-get.php
 * @return string a string.
 */
#[Pure]
function date_default_timezone_get(): string
{
}

/**
 * Returns time of sunrise for a given day and location
 * @link https://php.net/manual/en/function.date-sunrise.php
 * @param int $timestamp <p>
 * The timestamp of the day from which the sunrise
 * time is taken.
 * </p>
 * @param int $returnFormat [optional] <p>
 * <table>
 * format constants
 * <tr valign="top">
 * <td>constant</td>
 * <td>description</td>
 * <td>example</td>
 * </tr>
 * <tr valign="top">
 * <td>SUNFUNCS_RET_STRING</td>
 * <td>returns the result as string</td>
 * <td>16:46</td>
 * </tr>
 * <tr valign="top">
 * <td>SUNFUNCS_RET_DOUBLE</td>
 * <td>returns the result as float</td>
 * <td>16.78243132</td>
 * </tr>
 * <tr valign="top">
 * <td>SUNFUNCS_RET_TIMESTAMP</td>
 * <td>returns the result as integer (timestamp)</td>
 * <td>1095034606</td>
 * </tr>
 * </table>
 * </p>
 * @param float|null $latitude [optional] <p>
 * Defaults to North, pass in a negative value for South.
 * See also: date.default_latitude
 * </p>
 * @param float|null $longitude [optional] <p>
 * Defaults to East, pass in a negative value for West.
 * See also: date.default_longitude
 * </p>
 * @param float|null $zenith [optional] <p>
 * Default: date.sunrise_zenith
 * </p>
 * @param float|null $utcOffset [optional]
 * @return string|int|float|false the sunrise time in a specified format on
 * success or false on failure.
 * @deprecated 8.1
 * Use {@link date_sun_info} instead
 */
#[Pure(true)]
#[Deprecated(reason: 'in 8.1.  Use date_sun_info instead', since: '8.1')]
function date_sunrise(
    int $timestamp,
    int $returnFormat = SUNFUNCS_RET_STRING,
    null|float $latitude,
    null|float $longitude,
    null|float $zenith,
    null|float $utcOffset,
): string|int|float|false {
}

/**
 * Returns time of sunset for a given day and location
 * @link https://php.net/manual/en/function.date-sunset.php
 * @param int $timestamp <p>
 * The timestamp of the day from which the sunset
 * time is taken.
 * </p>
 * @param int $returnFormat [optional] <p>
 * <table>
 * format constants
 * <tr valign="top">
 * <td>constant</td>
 * <td>description</td>
 * <td>example</td>
 * </tr>
 * <tr valign="top">
 * <td>SUNFUNCS_RET_STRING</td>
 * <td>returns the result as string</td>
 * <td>16:46</td>
 * </tr>
 * <tr valign="top">
 * <td>SUNFUNCS_RET_DOUBLE</td>
 * <td>returns the result as float</td>
 * <td>16.78243132</td>
 * </tr>
 * <tr valign="top">
 * <td>SUNFUNCS_RET_TIMESTAMP</td>
 * <td>returns the result as integer (timestamp)</td>
 * <td>1095034606</td>
 * </tr>
 * </table>
 * </p>
 * @param float|null $latitude [optional] <p>
 * Defaults to North, pass in a negative value for South.
 * See also: date.default_latitude
 * </p>
 * @param float|null $longitude [optional] <p>
 * Defaults to East, pass in a negative value for West.
 * See also: date.default_longitude
 * </p>
 * @param float|null $zenith [optional] <p>
 * Default: date.sunset_zenith
 * </p>
 * @param float|null $utcOffset [optional]
 * @return string|int|float|false the sunset time in a specified format on
 * success or false on failure.
 */
#[Pure(true)]
#[Deprecated(reason: 'in 8.1.  Use date_sun_info instead', since: '8.1')]
function date_sunset(
    int $timestamp,
    int $returnFormat = SUNFUNCS_RET_STRING,
    null|float $latitude,
    null|float $longitude,
    null|float $zenith,
    null|float $utcOffset,
): string|int|float|false {
}

/**
 * Returns an array with information about sunset/sunrise and twilight begin/end
 * @link https://php.net/manual/en/function.date-sun-info.php
 * @param int $timestamp <p>
 * Timestamp.
 * </p>
 * @param float $latitude <p>
 * Latitude in degrees.
 * </p>
 * @param float $longitude <p>
 * Longitude in degrees.
 * </p>
 * @return array{
 *              sunrise: int|bool,
 *              sunset: int|bool,
 *              transit: int|bool,
 *              civil_twilight_begin: int|bool,
 *              civil_twilight_end: int|bool,
 *              nautical_twilight_begin: int|bool,
 *              nautical_twilight_end: int|bool,
 *              astronomical_twilight_begin: int|bool,
 *              astronomical_twilight_end: int|bool,
 *         }|false Returns array on success or <strong><code>false</code></strong> on failure. The structure of the array is detailed in the following list:
 * <table>
 * <tr><td>sunrise</td><td>The timestamp of the sunrise (zenith angle = 90°35&#039;).</td></tr>
 * <tr><td>sunset</td><td>The timestamp of the sunset (zenith angle = 90°35&#039;).</td></tr>
 * <tr><td>transit</td><td>The timestamp when the sun is at its zenith, i.e. has reached its topmost point.</td></tr>
 * <tr><td>civil_twilight_begin</td><td>The start of the civil dawn (zenith angle = 96°). It ends at <code>sunrise</code>.</td></tr>
 * <tr><td>civil_twilight_end</td><td>The end of the civil dusk (zenith angle = 96°). It starts at <code>sunset</code>.</td></tr>
 * <tr><td>nautical_twilight_begin</td><td>The start of the nautical dawn (zenith angle = 102°). It ends at <code>civil_twilight_begin</code>.</td></tr>
 * <tr><td>nautical_twilight_end</td><td>The end of the nautical dusk (zenith angle = 102°). It starts at <code>civil_twilight_end</code>.</td></tr>
 * <tr><td>astronomical_twilight_begin</td><td>The start of the astronomical dawn (zenith angle = 108°). It ends at <code>nautical_twilight_begin</code>.</td></tr>
 * <tr><td>astronomical_twilight_end</td><td>The end of the astronomical dusk (zenith angle = 108°). It starts at <code>nautical_twilight_end</code>.</td></tr>
 * </table>
 * <br>
 * The values of the array elements are either UNIX timestamps, <strong><code>false</code></strong> if the
 * sun is below the respective zenith for the whole day, or <strong><code>true</code></strong> if the sun is
 * above the respective zenith for the whole day.
 * @since 5.1.2
 */
#[Pure(true)]
#[LanguageLevelTypeAware(['8.0' => 'array'], default: 'array|false')]
#[ArrayShape([
    'sunrise' => 'int',
    'sunset' => 'int',
    'transit' => 'int',
    'civil_twilight_begin' => 'int',
    'civil_twilight_end' => 'int',
    'nautical_twilight_begin' => 'int',
    'nautical_twilight_end' => 'int',
    'astronomical_twilight_begin' => 'int',
    'astronomical_twilight_end' => 'int',
])]
function date_sun_info(int $timestamp, float $latitude, float $longitude): array|false
{
}
