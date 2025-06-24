<?php

use JetBrains\PhpStorm\ArrayShape;
use JetBrains\PhpStorm\Immutable;
use JetBrains\PhpStorm\Internal\LanguageLevelTypeAware;
use JetBrains\PhpStorm\Internal\PhpStormStubsElementAvailable;
use JetBrains\PhpStorm\Internal\TentativeType;
use JetBrains\PhpStorm\Pure;

interface DateTimeInterface
{
    public const ATOM = 'Y-m-d\TH:i:sP';
    public const COOKIE = 'l, d-M-Y H:i:s T';

    /**
     * @deprecated
     */
    public const ISO8601 = 'Y-m-d\TH:i:sO';
    public const ISO8601_EXPANDED = DATE_ISO8601_EXPANDED;
    public const RFC822 = 'D, d M y H:i:s O';
    public const RFC850 = 'l, d-M-y H:i:s T';
    public const RFC1036 = 'D, d M y H:i:s O';
    public const RFC1123 = 'D, d M Y H:i:s O';
    public const RFC2822 = 'D, d M Y H:i:s O';
    public const RFC3339 = 'Y-m-d\TH:i:sP';
    public const RFC3339_EXTENDED = 'Y-m-d\TH:i:s.vP';
    public const RFC7231 = 'D, d M Y H:i:s \G\M\T';
    public const RSS = 'D, d M Y H:i:s O';
    public const W3C = 'Y-m-d\TH:i:sP';

    /**
     * @param DateTimeInterface $targetObject
     * @param bool $absolute
     *
     * @return DateInterval
     */
    public function diff(DateTimeInterface $targetObject, bool $absolute = false): DateInterval;

    public function format(string $format): string;

    public function getOffset(): int;

    public function getTimestamp(): int;

    public function getTimezone(): DateTimeZone|false;

    public function __wakeup(): void;

    public function __serialize(): array;

    public function __unserialize(array $data): void;

    public function getMicrosecond(): int;

    public function setMicrosecond();
}

class DateTimeImmutable implements DateTimeInterface
{
    /* Methods */
    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * @link https://secure.php.net/manual/en/datetimeimmutable.construct.php
     * @param string $datetime [optional]
     * <p>A date/time string. Valid formats are explained in {@link https://secure.php.net/manual/en/datetime.formats.php Date and Time Formats}.</p>
     * <p>Enter <b>NULL</b> here to obtain the current time when using the <em>$timezone</em> parameter.</p>
     * @param null|DateTimeZone $timezone [optional] <p>
     * A {@link https://secure.php.net/manual/en/class.datetimezone.php DateTimeZone} object representing the timezone of <em>$datetime</em>.
     * </p>
     * <p>If <em>$timezone</em> is omitted, the current timezone will be used.</p>
     * <blockquote><p><b>Note</b>:</p><p>
     * The <em>$timezone</em> parameter and the current timezone are ignored when the <em>$datetime</em> parameter either
     * is a UNIX timestamp (e.g. <em>@946684800</em>) or specifies a timezone (e.g. <em>2010-01-28T15:00:00+02:00</em>).
     * </p></blockquote>
     * @throws Exception Emits Exception in case of an error.
     */
    #[PhpStormStubsElementAvailable(from: '5.5', to: '8.2')]
    public function __construct(string $datetime = 'now', DateTimeZone|null $timezone = null) {}

    /**
     * (PHP 8 &gt;=8.3.0)<br/>
     * @link https://secure.php.net/manual/en/datetimeimmutable.construct.php
     * @param string $datetime [optional]
     * <p>A date/time string. Valid formats are explained in {@link https://secure.php.net/manual/en/datetime.formats.php Date and Time Formats}.</p>
     * <p>Enter <b>NULL</b> here to obtain the current time when using the <em>$timezone</em> parameter.</p>
     * @param null|DateTimeZone $timezone [optional] <p>
     * A {@link https://secure.php.net/manual/en/class.datetimezone.php DateTimeZone} object representing the timezone of <em>$datetime</em>.
     * </p>
     * <p>If <em>$timezone</em> is omitted, the current timezone will be used.</p>
     * <blockquote><p><b>Note</b>:</p><p>
     * The <em>$timezone</em> parameter and the current timezone are ignored when the <em>$datetime</em> parameter either
     * is a UNIX timestamp (e.g. <em>@946684800</em>) or specifies a timezone (e.g. <em>2010-01-28T15:00:00+02:00</em>).
     * </p></blockquote>
     * @throws DateMalformedStringException Emits Exception in case of an error.
     */
    #[PhpStormStubsElementAvailable(from: '8.3')]
    public function __construct(string $datetime = 'now', DateTimeZone|null $timezone = null) {}

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Adds an amount of days, months, years, hours, minutes and seconds
     * @param DateInterval $interval
     * @return static
     * @link https://secure.php.net/manual/en/datetimeimmutable.add.php
     */
    public function add(DateInterval $interval): DateTimeImmutable
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Returns new DateTimeImmutable object formatted according to the specified format
     * @link https://secure.php.net/manual/en/datetimeimmutable.createfromformat.php
     * @param string $format
     * @param string $datetime
     * @param null|DateTimeZone $timezone [optional]
     * @return DateTimeImmutable|false
     */
    public static function createFromFormat(
        string $format,
        string $datetime,
        DateTimeZone|null $timezone = null,
    ): DateTimeImmutable|false {
    }

    /**
     * (PHP 5 &gt;=5.6.0)<br/>
     * Returns new DateTimeImmutable object encapsulating the given DateTime object
     * @link https://secure.php.net/manual/en/datetimeimmutable.createfrommutable.php
     * @param DateTime $object The mutable DateTime object that you want to convert to an immutable version. This object is not modified, but instead a new DateTimeImmutable object is created containing the same date time and timezone information.
     * @return DateTimeImmutable returns a new DateTimeImmutable instance.
     */
    #[LanguageLevelTypeAware(['8.2' => 'static'], default: 'DateTimeImmutable')]
    public static function createFromMutable(DateTime $object)
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Returns the warnings and errors
     * @link https://secure.php.net/manual/en/datetimeimmutable.getlasterrors.php
     * @return array|false Returns array containing info about warnings and errors.
     */
    #[ArrayShape(['warning_count' => 'int', 'warnings' => 'string[]', 'error_count' => 'int', 'errors' => 'string[]'])]
    public static function getLastErrors(): array|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Alters the timestamp
     * @link https://secure.php.net/manual/en/datetimeimmutable.modify.php
     * @param string $modifier <p>A date/time string. Valid formats are explained in
     * {@link https://secure.php.net/manual/en/datetime.formats.php Date and Time Formats}.</p>
     * @return static|false Returns the newly created object or false on failure.
     * Returns the {@link https://secure.php.net/manual/en/class.datetimeimmutable.php DateTimeImmutable} object for method chaining or <b>FALSE</b> on failure.
     */
    #[PhpStormStubsElementAvailable(from: '5.5', to: '8.2')]
    #[Pure]
    #[LanguageLevelTypeAware(['8.4' => 'DateTimeImmutable'], default: 'static|false')]
    public function modify(string $modifier)
    {
    }

    /**
     * (PHP 8 &gt;=8.3.0)<br/>
     * Alters the timestamp
     * @link https://secure.php.net/manual/en/datetimeimmutable.modify.php
     * @param string $modifier <p>A date/time string. Valid formats are explained in
     * {@link https://secure.php.net/manual/en/datetime.formats.php Date and Time Formats}.</p>
     * @return static|false Returns the newly created object or false on failure.
     * @throws DateMalformedStringException
     * Returns the {@link https://secure.php.net/manual/en/class.datetimeimmutable.php DateTimeImmutable} object for method chaining or <b>FALSE</b> on failure.
     */
    #[PhpStormStubsElementAvailable(from: '8.3')]
    #[Pure]
    #[LanguageLevelTypeAware(['8.4' => 'DateTimeImmutable'], default: 'DateTimeImmutable|false')]
    public function modify(string $modifier)
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * The __set_state handler
     * @link https://secure.php.net/manual/en/datetimeimmutable.set-state.php
     * @param array $array <p>Initialization array.</p>
     * @return DateTimeImmutable
     * Returns a new instance of a {@link https://secure.php.net/manual/en/class.datetimeimmutable.php DateTimeImmutable} object.
     */
    public static function __set_state(array $array)
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Sets the date
     * @link https://secure.php.net/manual/en/datetimeimmutable.setdate.php
     * @param int $year <p>Year of the date.</p>
     * @param int $month <p>Month of the date.</p>
     * @param int $day <p>Day of the date.</p>
     * @return static|false
     * Returns the {@link https://secure.php.net/manual/en/class.datetimeimmutable.php DateTimeImmutable} object for method chaining or <b>FALSE</b> on failure.
     */
    public function setDate(int $year, int $month, int $day): DateTimeImmutable
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Sets the ISO date
     * @link https://php.net/manual/en/class.datetimeimmutable.php
     * @param int $year <p>Year of the date.</p>
     * @param int $week <p>Week of the date.</p>
     * @param int $dayOfWeek [optional] <p>Offset from the first day of the week.</p>
     * @return static|false
     * Returns the {@link https://secure.php.net/manual/en/class.datetimeimmutable.php DateTimeImmutable} object for method chaining or <b>FALSE</b> on failure.
     */
    public function setISODate(int $year, int $week, int $dayOfWeek = 1): DateTimeImmutable
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Sets the time
     * @link https://secure.php.net/manual/en/datetimeimmutable.settime.php
     * @param int $hour <p> Hour of the time. </p>
     * @param int $minute <p> Minute of the time. </p>
     * @param int $second [optional] <p> Second of the time. </p>
     * @param int $microsecond [optional] <p> Microseconds of the time. Added since 7.1</p>
     * @return static|false
     * Returns the {@link https://secure.php.net/manual/en/class.datetimeimmutable.php DateTimeImmutable} object for method chaining or <b>FALSE</b> on failure.
     */
    public function setTime(int $hour, int $minute, int $second = 0, int $microsecond = 0): DateTimeImmutable
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Sets the date and time based on an Unix timestamp
     * @link https://secure.php.net/manual/en/datetimeimmutable.settimestamp.php
     * @param int $timestamp <p>Unix timestamp representing the date.</p>
     * @return static
     * Returns the {@link https://secure.php.net/manual/en/class.datetimeimmutable.php DateTimeImmutable} object for method chaining or <b>FALSE</b> on failure.
     */
    public function setTimestamp(int $timestamp): DateTimeImmutable
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Sets the time zone
     * @link https://secure.php.net/manual/en/datetimeimmutable.settimezone.php
     * @param DateTimeZone $timezone <p>
     * A {@link https://secure.php.net/manual/en/class.datetimezone.php DateTimeZone} object representing the
     * desired time zone.
     * </p>
     * @return static
     * Returns the {@link https://secure.php.net/manual/en/class.datetimeimmutable.php DateTimeImmutable} object for method chaining or <b>FALSE</b> on failure.
     */
    public function setTimezone(DateTimeZone $timezone): DateTimeImmutable
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Subtracts an amount of days, months, years, hours, minutes and seconds
     * @link https://secure.php.net/manual/en/datetimeimmutable.sub.php
     * @param DateInterval $interval <p>
     * A {@link https://secure.php.net/manual/en/class.dateinterval.php DateInterval} object
     * </p>
     * @return static
     * @throws DateInvalidOperationException
     * Returns the {@link https://secure.php.net/manual/en/class.datetimeimmutable.php DateTimeImmutable} object for method chaining or <b>FALSE</b> on failure.
     */
    public function sub(DateInterval $interval): DateTimeImmutable
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Returns the difference between two DateTime objects
     * @link https://secure.php.net/manual/en/datetime.diff.php
     * @param DateTimeInterface $targetObject <p>The date to compare to.</p>
     * @param bool $absolute [optional] <p>Should the interval be forced to be positive?</p>
     * @return DateInterval|false
     * The {@link https://secure.php.net/manual/en/class.dateinterval.php DateInterval} object representing the
     * difference between the two dates or <b>FALSE</b> on failure.
     */
    public function diff(DateTimeInterface $targetObject, bool $absolute = false): DateInterval
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Returns date formatted according to given format
     * @link https://secure.php.net/manual/en/datetime.format.php
     * @param string $format <p>
     * Format accepted by  {@link https://secure.php.net/manual/en/function.date.php date()}.
     * </p>
     * @return string
     * Returns the formatted date string on success or <b>FALSE</b> on failure.
     */
    public function format(string $format): string
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Returns the timezone offset
     * @return int
     * Returns the timezone offset in seconds from UTC on success
     * or <b>FALSE</b> on failure.
     */
    public function getOffset(): int
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Gets the Unix timestamp
     * @return int
     * Returns the Unix timestamp representing the date.
     */
    public function getTimestamp(): int
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Return time zone relative to given DateTime
     * @link https://secure.php.net/manual/en/datetime.gettimezone.php
     * @return DateTimeZone|false
     * Returns a {@link https://secure.php.net/manual/en/class.datetimezone.php DateTimeZone} object on success
     * or <b>FALSE</b> on failure.
     */
    public function getTimezone(): DateTimeZone|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * The __wakeup handler
     * @link https://secure.php.net/manual/en/datetime.wakeup.php
     * @return void Initializes a DateTime object.
     */
    public function __wakeup(): void
    {
    }

    /**
     * @param DateTimeInterface $object
     * @return DateTimeImmutable
     * @since 8.0
     */
    public static function createFromInterface(DateTimeInterface $object): DateTimeImmutable
    {
    }

    #[PhpStormStubsElementAvailable(from: '8.2')]
    public function __serialize(): array
    {
    }

    #[PhpStormStubsElementAvailable(from: '8.2')]
    public function __unserialize(array $data): void
    {
    }

    public static function createFromTimestamp(int|float $timestamp): static
    {
    }

    public function getMicrosecond(): int
    {
    }

    public function setMicrosecond(int $microsecond): static
    {
    }
}

/**
 * Representation of date and time.
 * @link https://php.net/manual/en/class.datetime.php
 */
class DateTime implements DateTimeInterface
{
    /**
     * (PHP 5 &gt;=5.2.0)<br/>
     * @link https://php.net/manual/en/datetime.construct.php
     * @param string $datetime [optional]
     * <p>A date/time string. Valid formats are explained in {@link https://php.net/manual/en/datetime.formats.php Date and Time Formats}.</p>
     * <p>
     * Enter <b>now</b> here to obtain the current time when using
     * the <em>$timezone</em> parameter.
     * </p>
     * @param null|DateTimeZone $timezone [optional] <p>
     * A {@link https://php.net/manual/en/class.datetimezone.php DateTimeZone} object representing the
     * timezone of <em>$datetime</em>.
     * </p>
     * <p>
     * If <em>$timezone</em> is omitted,
     * the current timezone will be used.
     * </p>
     * <blockquote><p><b>Note</b>:
     * </p><p>
     * The <em>$timezone</em> parameter
     * and the current timezone are ignored when the
     * <em>$time</em> parameter either
     * is a UNIX timestamp (e.g. <em>@946684800</em>)
     * or specifies a timezone
     * (e.g. <em>2010-01-28T15:00:00+02:00</em>).
     * </p> <p></p></blockquote>
     * @throws Exception Emits Exception in case of an error.
     */
    #[PhpStormStubsElementAvailable(from: '5.3', to: '8.2')]
    public function __construct(string $datetime = 'now', DateTimeZone|null $timezone = null) {}

    /**
     * (PHP 8 &gt;=8.3.0)<br/>
     * @link https://php.net/manual/en/datetime.construct.php
     * @param string $datetime [optional]
     * <p>A date/time string. Valid formats are explained in {@link https://php.net/manual/en/datetime.formats.php Date and Time Formats}.</p>
     * <p>
     * Enter <b>now</b> here to obtain the current time when using
     * the <em>$timezone</em> parameter.
     * </p>
     * @param null|DateTimeZone $timezone [optional] <p>
     * A {@link https://php.net/manual/en/class.datetimezone.php DateTimeZone} object representing the
     * timezone of <em>$datetime</em>.
     * </p>
     * <p>
     * If <em>$timezone</em> is omitted,
     * the current timezone will be used.
     * </p>
     * <blockquote><p><b>Note</b>:
     * </p><p>
     * The <em>$timezone</em> parameter
     * and the current timezone are ignored when the
     * <em>$time</em> parameter either
     * is a UNIX timestamp (e.g. <em>@946684800</em>)
     * or specifies a timezone
     * (e.g. <em>2010-01-28T15:00:00+02:00</em>).
     * </p> <p></p></blockquote>
     * @throws DateMalformedStringException Emits Exception in case of an error.
     */
    #[PhpStormStubsElementAvailable(from: '8.3')]
    public function __construct(string $datetime = 'now', DateTimeZone|null $timezone = null) {}

    /**
     * @return void
     * @link https://php.net/manual/en/datetime.wakeup.php
     */
    public function __wakeup(): void
    {
    }

    /**
     * Returns date formatted according to given format.
     * @param string $format
     * @return string
     * @link https://php.net/manual/en/datetime.format.php
     */
    public function format(string $format): string
    {
    }

    /**
     * Alter the timestamp of a DateTime object by incrementing or decrementing
     * in a format accepted by strtotime().
     * @param string $modifier A date/time string. Valid formats are explained in <a href="https://secure.php.net/manual/en/datetime.formats.php">Date and Time Formats</a>.
     * @return static|false Returns the DateTime object for method chaining or FALSE on failure.
     * @link https://php.net/manual/en/datetime.modify.php
     */
    #[PhpStormStubsElementAvailable(from: '5.3', to: '8.2')]
    #[LanguageLevelTypeAware(['8.4' => 'DateTime'], default: 'static|false')]
    public function modify(string $modifier)
    {
    }

    /**
     * Alter the timestamp of a DateTime object by incrementing or decrementing
     * in a format accepted by strtotime().
     * @param string $modifier A date/time string. Valid formats are explained in <a href="https://secure.php.net/manual/en/datetime.formats.php">Date and Time Formats</a>.
     * @return static|false Returns the DateTime object for method chaining or FALSE on failure.
     * @throws DateMalformedStringException
     * @link https://php.net/manual/en/datetime.modify.php
     */
    #[PhpStormStubsElementAvailable(from: '8.3')]
    #[LanguageLevelTypeAware(['8.4' => 'DateTime'], default: 'static|false')]
    public function modify(string $modifier)
    {
    }

    /**
     * Adds an amount of days, months, years, hours, minutes and seconds to a DateTime object
     * @param DateInterval $interval
     * @return static
     * @link https://php.net/manual/en/datetime.add.php
     */
    public function add(DateInterval $interval): DateTime
    {
    }

    /**
     * @param DateTimeImmutable $object
     * @return DateTime
     * @since 7.3
     */
    #[LanguageLevelTypeAware(['8.2' => 'static'], default: 'DateTime')]
    public static function createFromImmutable(DateTimeImmutable $object)
    {
    }

    /**
     * Subtracts an amount of days, months, years, hours, minutes and seconds from a DateTime object
     * @param DateInterval $interval
     * @return static
     * @link https://php.net/manual/en/datetime.sub.php
     * @throws DateInvalidOperationException
     */
    public function sub(DateInterval $interval): DateTime
    {
    }

    /**
     * Get the TimeZone associated with the DateTime
     * @return DateTimeZone|false
     * @link https://php.net/manual/en/datetime.gettimezone.php
     */
    public function getTimezone(): DateTimeZone|false
    {
    }

    /**
     * Set the TimeZone associated with the DateTime
     * @param DateTimeZone $timezone
     * @return static
     * @link https://php.net/manual/en/datetime.settimezone.php
     */
    public function setTimezone(#[LanguageLevelTypeAware(['8.0' => 'DateTimeZone'], default: '')]  $timezone): DateTime
    {
    }

    /**
     * Returns the timezone offset
     * @return int
     * @link https://php.net/manual/en/datetime.getoffset.php
     */
    public function getOffset(): int
    {
    }

    /**
     * Sets the current time of the DateTime object to a different time.
     * @param int $hour
     * @param int $minute
     * @param int $second
     * @param int $microsecond Added since 7.1
     * @return static
     * @link https://php.net/manual/en/datetime.settime.php
     */
    public function setTime(int $hour, int $minute, int $second = 0, int $microsecond = 0): DateTime
    {
    }

    /**
     * Sets the current date of the DateTime object to a different date.
     * @param int $year
     * @param int $month
     * @param int $day
     * @return static
     * @link https://php.net/manual/en/datetime.setdate.php
     */
    public function setDate(int $year, int $month, int $day): DateTime
    {
    }

    /**
     * Set a date according to the ISO 8601 standard - using weeks and day offsets rather than specific dates.
     * @param int $year
     * @param int $week
     * @param int $dayOfWeek
     * @return static
     * @link https://php.net/manual/en/datetime.setisodate.php
     */
    public function setISODate(int $year, int $week, int $dayOfWeek = 1): DateTime
    {
    }

    /**
     * Sets the date and time based on a Unix timestamp.
     * @param int $timestamp
     * @return static
     * @link https://php.net/manual/en/datetime.settimestamp.php
     */
    public function setTimestamp(int $timestamp): DateTime
    {
    }

    /**
     * Gets the Unix timestamp.
     * @return int
     * @link https://php.net/manual/en/datetime.gettimestamp.php
     */
    public function getTimestamp(): int
    {
    }

    /**
     * Returns the difference between two DateTime objects represented as a DateInterval.
     * @param DateTimeInterface $targetObject The date to compare to.
     * @param bool $absolute [optional] Whether to return absolute difference.
     * @return DateInterval|false The DateInterval object representing the difference between the two dates.
     * @link https://php.net/manual/en/datetime.diff.php
     */
    public function diff(DateTimeInterface $targetObject, bool $absolute = false): DateInterval
    {
    }

    /**
     * Parse a string into a new DateTime object according to the specified format
     * @param string $format Format accepted by date().
     * @param string $datetime String representing the time.
     * @param null|DateTimeZone $timezone A DateTimeZone object representing the desired time zone.
     * @return DateTime|false
     * @link https://php.net/manual/en/datetime.createfromformat.php
     */
    public static function createFromFormat(
        string $format,
        string $datetime,
        DateTimeZone|null $timezone = null,
    ): DateTime|false {
    }

    /**
     * Returns an array of warnings and errors found while parsing a date/time string
     * @return array|false
     * @link https://php.net/manual/en/datetime.getlasterrors.php
     */
    #[ArrayShape(['warning_count' => 'int', 'warnings' => 'string[]', 'error_count' => 'int', 'errors' => 'string[]'])]
    public static function getLastErrors(): array|false
    {
    }

    /**
     * @param array $array
     *
     * @return DateTime
     */
    public static function __set_state($array)
    {
    }

    public static function createFromInterface(DateTimeInterface $object): DateTime
    {
    }

    public function __serialize(): array
    {
    }

    public function __unserialize(array $data): void
    {
    }

    public static function createFromTimestamp(int|float $timestamp): static
    {
    }

    public function getMicrosecond(): int
    {
    }

    public function setMicrosecond(int $microsecond): static
    {
    }
}

class DateTimeZone
{
    public const AFRICA = 1;
    public const AMERICA = 2;
    public const ANTARCTICA = 4;
    public const ARCTIC = 8;
    public const ASIA = 16;
    public const ATLANTIC = 32;
    public const AUSTRALIA = 64;
    public const EUROPE = 128;
    public const INDIAN = 256;
    public const PACIFIC = 512;
    public const UTC = 1024;
    public const ALL = 2047;
    public const ALL_WITH_BC = 4095;
    public const PER_COUNTRY = 4096;

    /**
     * @throws DateInvalidTimeZoneException
     */
    public function __construct(string $timezone) {}

    public function getName(): string
    {
    }

    /**
     * @return false|array{
     *   country_code: string,
     *   latitude: float,
     *   longitude: float,
     *   comments: string,
     * }
     */
    public function getLocation(): array|false
    {
    }

    public function getOffset(DateTimeInterface $datetime): int
    {
    }

    public function getTransitions(int $timestampBegin = PHP_INT_MIN, int $timestampEnd = PHP_INT_MAX): array|false
    {
    }

    /**
     * @return array<string, list<array{dst: bool, offset: int, timezone_id: string|null}>>
     */
    public static function listAbbreviations(): array
    {
    }

    public static function listIdentifiers(
        int $timezoneGroup = DateTimeZone::ALL,
        string|null $countryCode = null,
    ): array {
    }

    public function __wakeup(): void
    {
    }

    public static function __set_state($an_array)
    {
    }

    public function __serialize(): array
    {
    }

    public function __unserialize(array $data): void
    {
    }
}

class DateInterval
{
    /**
     * @var int
     */
    public $y;

    /**
     * @var int
     */
    public $m;

    /**
     * @var int
     */
    public $d;

    /**
     * @var int
     */
    public $h;

    /**
     * @var int
     */
    public $i;

    /**
     * @var int
     */
    public $s;

    /**
     * @var float
     */
    public $f;

    /**
     * @var int
     */
    public $invert;

    /**
     * @var int|false
     */
    public $days;

    /**
     * @throws DateMalformedIntervalStringException
     */
    public function __construct(string $duration) {}

    public function format(string $format): string
    {
    }

    public static function createFromDateString(string $datetime): DateInterval
    {
    }

    public function __wakeup(): void
    {
    }

    public static function __set_state($an_array)
    {
    }

    public function __serialize(): array
    {
    }

    public function __unserialize(array $data): void
    {
    }
}

/**
 * @implements \IteratorAggregate<int, DateTimeInterface>
 */
class DatePeriod implements IteratorAggregate
{
    public const EXCLUDE_START_DATE = 1;

    public const INCLUDE_END_DATE = 2;

    /**
     * @readonly
     */
    public DateTimeInterface|null $start;

    /**
     * @readonly
     */
    public DateTimeInterface|null $current;

    /**
     * @readonly
     */
    public DateTimeInterface|null $end;

    /**
     * @readonly
     */
    public DateInterval|null $interval;

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

    /**
     * @param int $options
     */
    public function __construct(
        DateTimeInterface $start,
        DateInterval $interval,
        DateTimeInterface $end,
        $options = 0,
    ) {}

    /**
     * @param int $recurrences
     * @param int $options
     */
    public function __construct(DateTimeInterface $start, DateInterval $interval, $recurrences, $options = 0) {}

    /**
     * @param string $isostr
     * @param int $options
     *
     * @throws DateMalformedPeriodStringException
     */
    public function __construct($isostr, $options = 0) {}

    public function getDateInterval(): DateInterval
    {
    }

    public function getEndDate(): null|DateTimeInterface
    {
    }

    public function getStartDate(): DateTimeInterface
    {
    }

    public static function __set_state(array $array): DatePeriod
    {
    }

    public function __wakeup(): void
    {
    }

    public function getRecurrences(): null|int
    {
    }

    /**
     * @return \Iterator<int, DateTimeInterface>
     */
    public function getIterator(): Iterator
    {
    }

    public function __serialize(): array
    {
    }

    public function __unserialize(array $data): void
    {
    }

    public static function createFromISO8601String(string $specification, int $options = 0): static
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
