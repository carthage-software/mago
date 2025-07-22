<?php

class Collator
{
    public const DEFAULT_VALUE = -1;
    public const PRIMARY = 0;
    public const SECONDARY = 1;
    public const TERTIARY = 2;
    public const DEFAULT_STRENGTH = 2;
    public const QUATERNARY = 3;
    public const IDENTICAL = 15;
    public const OFF = 16;
    public const ON = 17;
    public const SHIFTED = 20;
    public const NON_IGNORABLE = 21;
    public const LOWER_FIRST = 24;
    public const UPPER_FIRST = 25;
    public const FRENCH_COLLATION = 0;
    public const ALTERNATE_HANDLING = 1;
    public const CASE_FIRST = 2;
    public const CASE_LEVEL = 3;
    public const NORMALIZATION_MODE = 4;
    public const STRENGTH = 5;
    public const HIRAGANA_QUATERNARY_MODE = 6;
    public const NUMERIC_COLLATION = 7;
    public const SORT_REGULAR = 0;
    public const SORT_STRING = 1;
    public const SORT_NUMERIC = 2;

    /**
     * @pure
     */
    public function __construct(string $locale) {}

    /**
     * @pure
     */
    public static function create(string $locale): null|Collator
    {
    }

    /**
     * @pure
     */
    public function compare(string $string1, string $string2): int|false
    {
    }

    /**
     * @param array<string> &$array
     */
    public function sort(array &$array, int $flags = 0): bool
    {
    }

    /**
     * @param array<string> &$array
     */
    public function sortWithSortKeys(array &$array): bool
    {
    }

    /**
     * @param array<string> &$array
     */
    public function asort(array &$array, int $flags = 0): bool
    {
    }

    /**
     * @pure
     */
    public function getAttribute(int $attribute): int|false
    {
    }

    public function setAttribute(int $attribute, int $value): bool
    {
    }

    /**
     * @pure
     */
    public function getStrength(): int
    {
    }

    public function setStrength(int $strength): bool
    {
    }

    /**
     * @pure
     */
    public function getErrorCode(): int|false
    {
    }

    /**
     * @pure
     */
    public function getLocale(int $type): string|false
    {
    }

    /**
     * @pure
     */
    public function getErrorMessage(): string|false
    {
    }

    /**
     * @pure
     */
    public function getSortKey(string $string): string|false
    {
    }
}

class NumberFormatter
{
    public const CURRENCY_ACCOUNTING = 12;
    public const PATTERN_DECIMAL = 0;
    public const DECIMAL = 1;
    public const CURRENCY = 2;
    public const PERCENT = 3;
    public const SCIENTIFIC = 4;
    public const SPELLOUT = 5;
    public const ORDINAL = 6;
    public const DURATION = 7;
    public const PATTERN_RULEBASED = 9;
    public const IGNORE = 0;
    public const DEFAULT_STYLE = 1;
    public const ROUND_CEILING = 0;
    public const ROUND_FLOOR = 1;
    public const ROUND_DOWN = 2;
    public const ROUND_UP = 3;
    public const ROUND_HALFEVEN = 4;
    public const ROUND_HALFDOWN = 5;
    public const ROUND_HALFUP = 6;
    public const PAD_BEFORE_PREFIX = 0;
    public const PAD_AFTER_PREFIX = 1;
    public const PAD_BEFORE_SUFFIX = 2;
    public const PAD_AFTER_SUFFIX = 3;
    public const PARSE_INT_ONLY = 0;
    public const GROUPING_USED = 1;
    public const DECIMAL_ALWAYS_SHOWN = 2;
    public const MAX_INTEGER_DIGITS = 3;
    public const MIN_INTEGER_DIGITS = 4;
    public const INTEGER_DIGITS = 5;
    public const MAX_FRACTION_DIGITS = 6;
    public const MIN_FRACTION_DIGITS = 7;
    public const FRACTION_DIGITS = 8;
    public const MULTIPLIER = 9;
    public const GROUPING_SIZE = 10;
    public const ROUNDING_MODE = 11;
    public const ROUNDING_INCREMENT = 12;
    public const FORMAT_WIDTH = 13;
    public const PADDING_POSITION = 14;
    public const SECONDARY_GROUPING_SIZE = 15;
    public const SIGNIFICANT_DIGITS_USED = 16;
    public const MIN_SIGNIFICANT_DIGITS = 17;
    public const MAX_SIGNIFICANT_DIGITS = 18;
    public const LENIENT_PARSE = 19;
    public const POSITIVE_PREFIX = 0;
    public const POSITIVE_SUFFIX = 1;
    public const NEGATIVE_PREFIX = 2;
    public const NEGATIVE_SUFFIX = 3;
    public const PADDING_CHARACTER = 4;
    public const CURRENCY_CODE = 5;
    public const DEFAULT_RULESET = 6;
    public const PUBLIC_RULESETS = 7;
    public const DECIMAL_SEPARATOR_SYMBOL = 0;
    public const GROUPING_SEPARATOR_SYMBOL = 1;
    public const PATTERN_SEPARATOR_SYMBOL = 2;
    public const PERCENT_SYMBOL = 3;
    public const ZERO_DIGIT_SYMBOL = 4;
    public const DIGIT_SYMBOL = 5;
    public const MINUS_SIGN_SYMBOL = 6;
    public const PLUS_SIGN_SYMBOL = 7;
    public const CURRENCY_SYMBOL = 8;
    public const INTL_CURRENCY_SYMBOL = 9;
    public const MONETARY_SEPARATOR_SYMBOL = 10;
    public const EXPONENTIAL_SYMBOL = 11;
    public const PERMILL_SYMBOL = 12;
    public const PAD_ESCAPE_SYMBOL = 13;
    public const INFINITY_SYMBOL = 14;
    public const NAN_SYMBOL = 15;
    public const SIGNIFICANT_DIGIT_SYMBOL = 16;
    public const MONETARY_GROUPING_SEPARATOR_SYMBOL = 17;
    public const TYPE_DEFAULT = 0;
    public const TYPE_INT32 = 1;
    public const TYPE_INT64 = 2;
    public const TYPE_DOUBLE = 3;
    public const TYPE_CURRENCY = 4;
    public const ROUND_TOWARD_ZERO = 2;
    public const ROUND_AWAY_FROM_ZERO = 3;
    public const ROUND_HALFODD = 8;

    /**
     * @pure
     */
    public function __construct(string $locale, int $style, string|null $pattern = null) {}

    public static function create(string $locale, int $style, string|null $pattern = null): null|NumberFormatter
    {
    }

    public function format(int|float $num, int $type = 0): string|false
    {
    }

    /**
     * @param-out int $offset
     */
    public function parse(string $string, int $type = NumberFormatter::TYPE_DOUBLE, &$offset = null): int|float|false
    {
    }

    /**
     * @pure
     */
    public function formatCurrency(float $amount, string $currency): string|false
    {
    }

    /**
     * @param-out string $currency
     * @param-out int $offset
     */
    public function parseCurrency(string $string, &$currency, &$offset = null): float|false
    {
    }

    public function setAttribute(int $attribute, int|float $value): bool
    {
    }

    /**
     * @pure
     */
    public function getAttribute(int $attribute): int|float|false
    {
    }

    public function setTextAttribute(int $attribute, string $value): bool
    {
    }

    /**
     * @pure
     */
    public function getTextAttribute(int $attribute): string|false
    {
    }

    public function setSymbol(int $symbol, string $value): bool
    {
    }

    /**
     * @pure
     */
    public function getSymbol(int $symbol): string|false
    {
    }

    public function setPattern(string $pattern): bool
    {
    }

    /**
     * @return string|false
     *
     * @pure
     */
    public function getPattern(): string|false
    {
    }

    /**
     * @pure
     */
    public function getLocale(int $type = 0): string|false
    {
    }

    /**
     * @pure
     */
    public function getErrorCode(): int
    {
    }

    /**
     * @pure
     */
    public function getErrorMessage(): string
    {
    }
}

class Normalizer
{
    public const NFKC_CF = 48;
    public const FORM_KC_CF = 48;
    public const OPTION_DEFAULT = '';
    public const FORM_D = 4;
    public const NFD = 4;
    public const FORM_KD = 8;
    public const NFKD = 8;
    public const FORM_C = 16;
    public const NFC = 16;
    public const FORM_KC = 32;
    public const NFKC = 32;

    public static function normalize(string $string, int $form = Normalizer::FORM_C): string|false
    {
    }

    public static function isNormalized(string $string, int $form = Normalizer::FORM_C): bool
    {
    }

    public static function getRawDecomposition(string $string, int $form = 16): null|string
    {
    }
}

class Locale
{
    public const ACTUAL_LOCALE = 0;
    public const VALID_LOCALE = 1;
    public const DEFAULT_LOCALE = null;
    public const LANG_TAG = 'language';
    public const EXTLANG_TAG = 'extlang';
    public const SCRIPT_TAG = 'script';
    public const REGION_TAG = 'region';
    public const VARIANT_TAG = 'variant';
    public const GRANDFATHERED_LANG_TAG = 'grandfathered';
    public const PRIVATE_TAG = 'private';

    public static function getDefault(): string
    {
    }

    public static function setDefault(string $locale): bool
    {
    }

    public static function getPrimaryLanguage(string $locale): null|string
    {
    }

    public static function getScript(string $locale): null|string
    {
    }

    public static function getRegion(string $locale): null|string
    {
    }

    public static function getKeywords(string $locale): array|false|null
    {
    }

    public static function getDisplayScript(string $locale, string|null $displayLocale = null): string|false
    {
    }

    public static function getDisplayRegion(string $locale, string|null $displayLocale = null): string|false
    {
    }

    public static function getDisplayName(string $locale, string|null $displayLocale = null): string|false
    {
    }

    public static function getDisplayLanguage(string $locale, string|null $displayLocale = null): string|false
    {
    }

    public static function getDisplayVariant(string $locale, string|null $displayLocale = null): string|false
    {
    }

    public static function composeLocale(array $subtags): string|false
    {
    }

    public static function parseLocale(string $locale): null|array
    {
    }

    public static function getAllVariants(string $locale): null|array
    {
    }

    public static function filterMatches(string $languageTag, string $locale, bool $canonicalize = false): null|bool
    {
    }

    public static function lookup(
        array $languageTag,
        string $locale,
        bool $canonicalize = false,
        string|null $defaultLocale = null,
    ): null|string {
    }

    public static function canonicalize(string $locale): null|string
    {
    }

    public static function acceptFromHttp(string $header): string|false
    {
    }
}

class MessageFormatter
{
    /**
     * @throws IntlException
     *
     * @pure
     */
    public function __construct(string $locale, string $pattern) {}

    public static function create(string $locale, string $pattern): null|MessageFormatter
    {
    }

    /**
     * @pure
     */
    public function format(array $values): string|false
    {
    }

    public static function formatMessage(string $locale, string $pattern, array $values): string|false
    {
    }

    /**
     * @pure
     */
    public function parse(string $string): array|false
    {
    }

    public static function parseMessage(string $locale, string $pattern, string $message): array|false
    {
    }

    public function setPattern(string $pattern): bool
    {
    }

    /**
     * @pure
     */
    public function getPattern(): string|false
    {
    }

    /**
     * @pure
     */
    public function getLocale(): string
    {
    }

    /**
     * @pure
     */
    public function getErrorCode(): int
    {
    }

    /**
     * @pure
     */
    public function getErrorMessage(): string
    {
    }
}

class IntlDateFormatter
{
    public const FULL = 0;
    public const LONG = 1;
    public const MEDIUM = 2;
    public const SHORT = 3;
    public const NONE = -1;
    public const GREGORIAN = 1;
    public const TRADITIONAL = 0;
    public const RELATIVE_FULL = 128;
    public const RELATIVE_LONG = 129;
    public const RELATIVE_MEDIUM = 130;
    public const RELATIVE_SHORT = 131;
    public const PATTERN = -2;

    /**
     * @pure
     */
    public function __construct(
        string|null $locale,
        int $dateType = 0,
        int $timeType = 0,
        $timezone = null,
        $calendar = null,
        string|null $pattern = null,
    ) {}

    public static function create(
        string|null $locale,
        int $dateType = 0,
        int $timeType = 0,
        $timezone = null,
        IntlCalendar|int|null $calendar = null,
        string|null $pattern = null,
    ): null|IntlDateFormatter {
    }

    /**
     * @pure
     */
    public function getDateType(): int|false
    {
    }

    /**
     * @pure
     */
    public function getTimeType(): int|false
    {
    }

    /**
     * @pure
     */
    public function getCalendar(): int|false
    {
    }

    public function setCalendar(IntlCalendar|int|null $calendar): bool
    {
    }

    /**
     * @pure
     */
    public function getTimeZoneId(): string|false
    {
    }

    /**
     * @pure
     */
    public function getCalendarObject(): IntlCalendar|false|null
    {
    }

    /**
     * @pure
     */
    public function getTimeZone(): IntlTimeZone|false
    {
    }

    /**
     * @return bool|null
     */
    public function setTimeZone($timezone)
    {
    }

    public function setPattern(string $pattern): bool
    {
    }

    /**
     * @pure
     */
    public function getPattern(): string|false
    {
    }

    /**
     * @pure
     */
    public function getLocale(int $type = 0): string|false
    {
    }

    public function setLenient(bool $lenient): void
    {
    }

    /**
     * @pure
     */
    public function isLenient(): bool
    {
    }

    /**
     * @param DateTimeInterface|IntlCalendar|array{
     *   0?: int,
     *   1?: int,
     *   2?: int,
     *   3?: int,
     *   4?: int,
     *   5?: int,
     *   6?: int,
     *   7?: int,
     *   8?: int,
     *   tm_hour?: int,
     *   tm_isdst?: int,
     *   tm_mday?: int,
     *   tm_min?: int,
     *   tm_mon?: int,
     *   tm_sec?: int,
     *   tm_wday?: int,
     *   tm_yday?: int,
     *   tm_year?: int,
     * }|float|int|string $datetime
     */
    public function format(IntlCalendar|DateTimeInterface|array|string|int|float $datetime): string|false
    {
    }

    /**
     * @param IntlCalendar|DateTimeInterface $datetime
     * @param null|int|string|array<string|int> $format
     */
    public static function formatObject($datetime, $format = null, string|null $locale = null): string|false
    {
    }

    /**
     * @param-out int $offset
     */
    public function parse(string $string, &$offset = null): int|float|false
    {
    }

    /**
     * @param-out int $offset
     */
    public function localtime(string $string, &$offset = null): array|false
    {
    }

    /**
     * @pure
     */
    public function getErrorCode(): int
    {
    }

    /**
     * @pure
     */
    public function getErrorMessage(): string
    {
    }

    public function parseToCalendar(string $string, &$offset = null): int|float|false
    {
    }
}

class ResourceBundle implements IteratorAggregate, Countable
{
    /**
     * @pure
     */
    public function __construct(string|null $locale, string|null $bundle, bool $fallback = true) {}

    public static function create(string|null $locale, string|null $bundle, bool $fallback = true): null|ResourceBundle
    {
    }

    /**
     * @return ResourceBundle|array|string|int|null
     *
     * @pure
     */
    public function get(string|int $index, bool $fallback = true)
    {
    }

    /**
     * @return int<0, max>
     *
     * @pure
     */
    public function count(): int
    {
    }

    public static function getLocales(string $bundle): array|false
    {
    }

    /**
     * @pure
     */
    public function getErrorCode(): int
    {
    }

    /**
     * @pure
     */
    public function getErrorMessage(): string
    {
    }

    /**
     * @pure
     */
    public function getIterator(): Iterator
    {
    }
}

class Transliterator
{
    public const FORWARD = 0;
    public const REVERSE = 1;

    public readonly string $id;

    final private function __construct() {}

    public static function create(string $id, int $direction = 0): null|Transliterator
    {
    }

    public static function createFromRules(string $rules, int $direction = 0): null|Transliterator
    {
    }

    /**
     * @pure
     */
    public function createInverse(): null|Transliterator
    {
    }

    public static function listIDs(): array|false
    {
    }

    /**
     * @pure
     */
    public function transliterate(string $string, int $start = 0, int $end = -1): string|false
    {
    }

    /**
     * @pure
     */
    public function getErrorCode(): int|false
    {
    }

    /**
     * @pure
     */
    public function getErrorMessage(): string|false
    {
    }
}

class Spoofchecker
{
    public const SINGLE_SCRIPT_CONFUSABLE = 1;
    public const MIXED_SCRIPT_CONFUSABLE = 2;
    public const WHOLE_SCRIPT_CONFUSABLE = 4;
    public const ANY_CASE = 8;
    public const SINGLE_SCRIPT = 16;
    public const INVISIBLE = 32;
    public const CHAR_LIMIT = 64;
    public const ASCII = 268435456;
    public const HIGHLY_RESTRICTIVE = 805306368;
    public const MODERATELY_RESTRICTIVE = 1073741824;
    public const MINIMALLY_RESTRICTIVE = 1342177280;
    public const UNRESTRICTIVE = 1610612736;
    public const SINGLE_SCRIPT_RESTRICTIVE = 536870912;
    public const MIXED_NUMBERS = 1;
    public const HIDDEN_OVERLAY = 2;
    public const IGNORE_SPACE = 1;
    public const CASE_INSENSITIVE = 2;
    public const ADD_CASE_MAPPINGS = 4;
    public const SIMPLE_CASE_INSENSITIVE = 6;

    /**
     * @pure
     */
    public function __construct() {}

    /**
     * @param-out string $errorCode
     */
    public function isSuspicious(string $string, &$errorCode = null): bool
    {
    }

    /**
     * @param-out int $errorCode
     */
    public function areConfusable(string $string1, string $string2, &$errorCode = null): bool
    {
    }

    public function setAllowedLocales(string $locales): void
    {
    }

    public function setChecks(int $checks): void
    {
    }

    public function setRestrictionLevel(int $level): void
    {
    }

    public function setAllowedChars(string $pattern, int $patternOptions = 0): void
    {
    }
}

class IntlGregorianCalendar extends IntlCalendar
{
    /**
     * @param int $timezoneOrYear
     * @param int $localeOrMonth
     * @param int $day
     * @param int $hour
     * @param int $minute
     * @param int $second
     */
    public function __construct($timezoneOrYear, $localeOrMonth, $day, $hour, $minute, $second) {}

    /**
     * @param mixed $timeZone
     * @param string $locale
     * @return IntlGregorianCalendar
     */
    public static function createInstance($timeZone = null, $locale = null)
    {
    }

    /**
     * @param float $timestamp
     */
    public function setGregorianChange(float $timestamp): bool
    {
    }

    /**
     * @return float
     *
     * @pure
     */
    public function getGregorianChange(): float
    {
    }

    /**
     * @param int $year
     * @return bool
     *
     * @pure
     */
    public function isLeapYear(int $year): bool
    {
    }

    public static function createFromDate(int $year, int $month, int $dayOfMonth): static
    {
    }

    public static function createFromDateTime(
        int $year,
        int $month,
        int $dayOfMonth,
        int $hour,
        int $minute,
        null|int $second = null,
    ): static {
    }
}

class IntlCalendar
{
    public const FIELD_ERA = 0;
    public const FIELD_YEAR = 1;
    public const FIELD_MONTH = 2;
    public const FIELD_WEEK_OF_YEAR = 3;
    public const FIELD_WEEK_OF_MONTH = 4;
    public const FIELD_DATE = 5;
    public const FIELD_DAY_OF_YEAR = 6;
    public const FIELD_DAY_OF_WEEK = 7;
    public const FIELD_DAY_OF_WEEK_IN_MONTH = 8;
    public const FIELD_AM_PM = 9;
    public const FIELD_HOUR = 10;
    public const FIELD_HOUR_OF_DAY = 11;
    public const FIELD_MINUTE = 12;
    public const FIELD_SECOND = 13;
    public const FIELD_MILLISECOND = 14;
    public const FIELD_ZONE_OFFSET = 15;
    public const FIELD_DST_OFFSET = 16;
    public const FIELD_YEAR_WOY = 17;
    public const FIELD_DOW_LOCAL = 18;
    public const FIELD_EXTENDED_YEAR = 19;
    public const FIELD_JULIAN_DAY = 20;
    public const FIELD_MILLISECONDS_IN_DAY = 21;
    public const FIELD_IS_LEAP_MONTH = 22;
    public const FIELD_FIELD_COUNT = 23;
    public const FIELD_DAY_OF_MONTH = 5;
    public const DOW_SUNDAY = 1;
    public const DOW_MONDAY = 2;
    public const DOW_TUESDAY = 3;
    public const DOW_WEDNESDAY = 4;
    public const DOW_THURSDAY = 5;
    public const DOW_FRIDAY = 6;
    public const DOW_SATURDAY = 7;
    public const DOW_TYPE_WEEKDAY = 0;
    public const DOW_TYPE_WEEKEND = 1;
    public const DOW_TYPE_WEEKEND_OFFSET = 2;
    public const DOW_TYPE_WEEKEND_CEASE = 3;
    public const WALLTIME_FIRST = 1;
    public const WALLTIME_LAST = 0;
    public const WALLTIME_NEXT_VALID = 2;

    public function add(int $field, int $value): bool
    {
    }

    /**
     * @pure
     */
    public function after(IntlCalendar $other): bool
    {
    }

    /**
     * @pure
     */
    public function before(IntlCalendar $other): bool
    {
    }

    public function clear(null|int $field = null): bool
    {
    }

    private function __construct() {}

    public static function createInstance($timezone = null, string|null $locale = null): null|IntlCalendar
    {
    }

    /**
     * @pure
     */
    public function equals(IntlCalendar $other): bool
    {
    }

    /**
     * @pure
     */
    public function fieldDifference(float $timestamp, int $field): int|false
    {
    }

    public static function fromDateTime(DateTime|string $datetime, string|null $locale): null|IntlCalendar
    {
    }

    /**
     * @return ($field is IntlCalendar::FIELD_* ? int : false)
     *
     * @pure
     */
    public function get(int $field): int|false
    {
    }

    /**
     * @return ($field is IntlCalendar::FIELD_* ? int : false)
     *
     * @pure
     */
    public function getActualMaximum(int $field): int|false
    {
    }

    /**
     * @return ($field is IntlCalendar::FIELD_* ? int : false)
     *
     * @pure
     */
    public function getActualMinimum(int $field): int|false
    {
    }

    /**
     * @return string[]
     */
    public static function getAvailableLocales(): array
    {
    }

    /**
     * @return (
     *   $dayOfWeek is IntlCalendar::DOW_MONDAY|IntlCalendar::DOW_TUESDAY|IntlCalendar::DOW_WEDNESDAY|IntlCalendar::DOW_THURSDAY|IntlCalendar::DOW_FRIDAY|IntlCalendar::DOW_SATURDAY|IntlCalendar::DOW_SUNDAY
     *   ? int
     *   : false
     * )
     *
     * @pure
     */
    public function getDayOfWeekType(int $dayOfWeek): int|false
    {
    }

    /**
     * @return int|false An ICU error code indicating either success, failure or a warning.
     *
     * @pure
     */
    public function getErrorCode(): int|false
    {
    }

    /**
     * @pure
     */
    public function getErrorMessage(): string|false
    {
    }

    /**
     * @pure
     */
    public function getFirstDayOfWeek(): int|false
    {
    }

    /**
     * @return ($field is IntlCalendar::FIELD_* ? int : false)
     *
     * @pure
     */
    public function getGreatestMinimum(int $field): int|false
    {
    }

    public static function getKeywordValuesForLocale(
        string $keyword,
        string $locale,
        bool $onlyCommon,
    ): IntlIterator|false {
    }

    /**
     * @return ($field is IntlCalendar::FIELD_* ? int : false)
     *
     * @pure
     */
    public function getLeastMaximum(int $field): int|false
    {
    }

    /**
     * @pure
     */
    public function getLocale(int $type): string|false
    {
    }

    /**
     * @return ($field is IntlCalendar::FIELD_* ? int : false)
     *
     * @pure
     */
    public function getMaximum(int $field): int|false
    {
    }

    /**
     * @pure
     */
    public function getMinimalDaysInFirstWeek(): int|false
    {
    }

    /**
     * @return ($field is IntlCalendar::FIELD_* ? int : false)
     *
     * @pure
     */
    public function getMinimum(int $field): int|false
    {
    }

    public static function getNow(): float
    {
    }

    /**
     * @pure
     */
    public function getRepeatedWallTimeOption(): int
    {
    }

    /**
     * @pure
     */
    public function getSkippedWallTimeOption(): int
    {
    }

    /**
     * @pure
     */
    public function getTime(): float|false
    {
    }

    /**
     * @pure
     */
    public function getTimeZone(): IntlTimeZone|false
    {
    }

    /**
     * @pure
     */
    public function getType(): string
    {
    }

    /**
     * @return (
     *   $dayOfWeek is IntlCalendar::DOW_MONDAY|IntlCalendar::DOW_TUESDAY|IntlCalendar::DOW_WEDNESDAY|IntlCalendar::DOW_THURSDAY|IntlCalendar::DOW_FRIDAY|IntlCalendar::DOW_SATURDAY|IntlCalendar::DOW_SUNDAY
     *     ? int
     *     : false
     * )
     *
     * @pure
     */
    public function getWeekendTransition(int $dayOfWeek): int|false
    {
    }

    /**
     * @pure
     */
    public function inDaylightTime(): bool
    {
    }

    /**
     * @pure
     */
    public function isEquivalentTo(IntlCalendar $other): bool
    {
    }

    /**
     * @pure
     */
    public function isLenient(): bool
    {
    }

    /**
     * @pure
     */
    public function isWeekend(float|null $timestamp = null): bool
    {
    }

    public function roll(int $field, $value): bool
    {
    }

    public function isSet(int $field): bool
    {
    }

    /**
     * @param int $year
     * @param int $month
     * @param null|int $dayOfMonth
     * @param null|int $hour
     * @param null|int $minute
     * @param null|int $second
     *
     * @return bool
     */
    public function set($year, $month, $dayOfMonth = null, $hour = null, $minute = null, $second = null)
    {
    }

    /**
     * @param int $field
     * @param int $value
     *
     * @return bool
     */
    public function set($field, $value)
    {
    }

    public function setFirstDayOfWeek(int $dayOfWeek): bool
    {
    }

    public function setLenient(bool $lenient): bool
    {
    }

    public function setRepeatedWallTimeOption(int $option): bool
    {
    }

    public function setSkippedWallTimeOption(int $option): bool
    {
    }

    public function setTime(float $timestamp): bool
    {
    }

    public function setTimeZone($timezone): bool
    {
    }

    /**
     * @pure
     */
    public function toDateTime(): DateTime|false
    {
    }

    public function setMinimalDaysInFirstWeek(int $days): bool
    {
    }

    public function setDate(int $year, int $month, int $dayOfMonth): void
    {
    }

    public function setDateTime(
        int $year,
        int $month,
        int $dayOfMonth,
        int $hour,
        int $minute,
        null|int $second = null,
    ): void {
    }
}

class IntlIterator implements Iterator
{
    public function current(): mixed
    {
    }

    public function key(): mixed
    {
    }

    public function next(): void
    {
    }

    public function rewind(): void
    {
    }

    public function valid(): bool
    {
    }
}

class IntlException extends Exception
{
}

class IntlTimeZone
{
    public const DISPLAY_SHORT = 1;
    public const DISPLAY_LONG = 2;
    public const DISPLAY_SHORT_GENERIC = 3;
    public const DISPLAY_LONG_GENERIC = 4;
    public const DISPLAY_SHORT_GMT = 5;
    public const DISPLAY_LONG_GMT = 6;
    public const DISPLAY_SHORT_COMMONLY_USED = 7;
    public const DISPLAY_GENERIC_LOCATION = 8;
    public const TYPE_ANY = 0;
    public const TYPE_CANONICAL = 1;
    public const TYPE_CANONICAL_LOCATION = 2;

    private function __construct() {}

    public static function countEquivalentIDs(string $timezoneId): int|false
    {
    }

    public static function createDefault(): IntlTimeZone
    {
    }

    public static function createEnumeration(mixed $countryOrRawOffset): IntlIterator|false
    {
    }

    public static function createTimeZone(string $timezoneId): null|IntlTimeZone
    {
    }

    public static function createTimeZoneIDEnumeration(
        int $type,
        string|null $region = null,
        null|int $rawOffset = null,
    ): IntlIterator|false {
    }

    public static function fromDateTimeZone(DateTimeZone $timezone): null|IntlTimeZone
    {
    }

    /**
     * @param-out bool $isSystemId
     */
    public static function getCanonicalID(string $timezoneId, &$isSystemId): string|false
    {
    }

    /**
     * @pure
     */
    public function getDisplayName(bool $dst = false, int $style = 2, string|null $locale): string|false
    {
    }

    /**
     * @pure
     */
    public function getDSTSavings(): int
    {
    }

    public static function getEquivalentID(string $timezoneId, int $offset): string|false
    {
    }

    /**
     * @pure
     */
    public function getErrorCode(): int|false
    {
    }

    /**
     * @pure
     */
    public function getErrorMessage(): string|false
    {
    }

    public static function getGMT(): IntlTimeZone
    {
    }

    /**
     * @pure
     */
    public function getID(): string|false
    {
    }

    /**
     * @param-out int $rawOffset
     * @param-out int $dstOffset
     */
    public function getOffset(float $timestamp, bool $local, &$rawOffset, &$dstOffset): bool
    {
    }

    /**
     * @pure
     */
    public function getRawOffset(): int
    {
    }

    public static function getRegion(string $timezoneId): string|false
    {
    }

    public static function getTZDataVersion(): string|false
    {
    }

    public static function getUnknown(): IntlTimeZone
    {
    }

    public static function getWindowsID(string $timezoneId): string|false
    {
    }

    public static function getIDForWindowsID(string $timezoneId, null|string $region = null): string|false
    {
    }

    /**
     * @pure
     */
    public function hasSameRules(IntlTimeZone $other): bool
    {
    }

    /**
     * @pure
     */
    public function toDateTimeZone(): DateTimeZone|false
    {
    }

    /**
     * @return bool
     */
    public function useDaylightTime(): bool
    {
    }

    public static function getIanaID(string $timezoneId): string|false
    {
    }
}

/**
 * @pure
 */
function collator_create(string $locale): null|Collator
{
}

/**
 * @pure
 */
function collator_compare(Collator $object, string $string1, string $string2): int|false
{
}

/**
 * @pure
 */
function collator_get_attribute(Collator $object, int $attribute): int|false
{
}

function collator_set_attribute(Collator $object, int $attribute, int $value): bool
{
}

/**
 * @pure
 */
function collator_get_strength(Collator $object): int
{
}

function collator_set_strength(Collator $object, int $strength): bool
{
}

/**
 * @param-out string[] $array
 */
function collator_sort(Collator $object, array &$array, int $flags = 0): bool
{
}

/**
 * @param-out string[] $array
 */
function collator_sort_with_sort_keys(Collator $object, array &$array): bool
{
}

/**
 * @param-out string[] $array
 */
function collator_asort(Collator $object, array &$array, int $flags = 0): bool
{
}

/**
 * @pure
 */
function collator_get_locale(Collator $object, int $type): string|false
{
}

function collator_get_error_code(Collator $object): int|false
{
}

/**
 * @pure
 */
function collator_get_error_message(Collator $object): string|false
{
}

/**
 * @pure
 */
function collator_get_sort_key(Collator $object, string $string): string|false
{
}

/**
 * @pure
 */
function numfmt_create(string $locale, int $style, string|null $pattern = null): null|NumberFormatter
{
}

/**
 * @pure
 */
function numfmt_format(NumberFormatter $formatter, int|float $num, int $type = 0): string|false
{
}

/**
 * @param-out int $offset
 *
 * @pure
 */
function numfmt_parse(
    NumberFormatter $formatter,
    string $string,
    int $type = NumberFormatter::TYPE_DOUBLE,
    &$offset = null,
): int|float|false {
}

/**
 * @pure
 */
function numfmt_format_currency(NumberFormatter $formatter, float $amount, string $currency): string|false
{
}

/**
 * @param-out string $currency
 * @param-out int $offset
 *
 * @return float|false
 */
function numfmt_parse_currency(NumberFormatter $formatter, string $string, &$currency, &$offset = null): float|false
{
}

function numfmt_set_attribute(NumberFormatter $formatter, int $attribute, int|float $value): bool
{
}

/**
 * @pure
 */
function numfmt_get_attribute(NumberFormatter $formatter, int $attribute): int|float|false
{
}

function numfmt_set_text_attribute(NumberFormatter $formatter, int $attribute, string $value): bool
{
}

/**
 * @pure
 */
function numfmt_get_text_attribute(NumberFormatter $formatter, int $attribute): string|false
{
}

function numfmt_set_symbol(NumberFormatter $formatter, int $symbol, string $value): bool
{
}

/**
 * @pure
 */
function numfmt_get_symbol(NumberFormatter $formatter, int $symbol): string|false
{
}

function numfmt_set_pattern(NumberFormatter $formatter, string $pattern): bool
{
}

/**
 * @pure
 */
function numfmt_get_pattern(NumberFormatter $formatter): string|false
{
}

/**
 * @pure
 */
function numfmt_get_locale(NumberFormatter $formatter, int $type = 0): string|false
{
}

function numfmt_get_error_code(NumberFormatter $formatter): int
{
}

function numfmt_get_error_message(NumberFormatter $formatter): string
{
}

/**
 * @pure
 */
function normalizer_normalize(string $string, int $form = Normalizer::FORM_C): string|false
{
}

/**
 * @pure
 */
function normalizer_is_normalized(string $string, int $form = Normalizer::FORM_C): bool
{
}

/**
 * @pure
 */
function locale_get_default(): string
{
}

function locale_set_default(string $locale): bool
{
}

/**
 * @pure
 */
function locale_get_primary_language(string $locale): null|string
{
}

/**
 * @pure
 */
function locale_get_script(string $locale): null|string
{
}

/**
 * @pure
 */
function locale_get_region(string $locale): null|string
{
}

/**
 * @pure
 */
function locale_get_keywords(string $locale): array|false|null
{
}

/**
 * @pure
 */
function locale_get_display_script(string $locale, null|string $displayLocale = null): string|false
{
}

/**
 * @pure
 */
function locale_get_display_region(string $locale, null|string $displayLocale = null): string|false
{
}

/**
 * @pure
 */
function locale_get_display_name(string $locale, null|string $displayLocale = null): string|false
{
}

/**
 * @pure
 */
function locale_get_display_language(string $locale, null|string $displayLocale = null): string|false
{
}

/**
 * @pure
 */
function locale_get_display_variant(string $locale, null|string $displayLocale = null): string|false
{
}

/**
 * @param string[] $subtags
 *
 * @pure
 */
function locale_compose(array $subtags): string|false
{
}

/**
 * @return string[]|null
 *
 * @pure
 */
function locale_parse(string $locale): null|array
{
}

/**
 * @pure
 */
function locale_get_all_variants(string $locale): null|array
{
}

/**
 * @pure
 */
function locale_filter_matches(string $languageTag, string $locale, bool $canonicalize = false): null|bool
{
}

/**
 * @pure
 */
function locale_canonicalize(string $locale): null|string
{
}

/**
 * @param string[] $languageTag
 *
 * @pure
 */
function locale_lookup(
    array $languageTag,
    string $locale,
    bool $canonicalize = false,
    null|string $defaultLocale = null,
): null|string {
}

/**
 * @pure
 */
function locale_accept_from_http(string $header): string|false
{
}

/**
 * @pure
 */
function msgfmt_create(string $locale, string $pattern): null|MessageFormatter
{
}

/**
 * @pure
 */
function msgfmt_format(MessageFormatter $formatter, array $values): string|false
{
}

/**
 * @pure
 */
function msgfmt_format_message(string $locale, string $pattern, array $values): string|false
{
}

/**
 * @pure
 */
function msgfmt_parse(MessageFormatter $formatter, string $string): array|false
{
}

/**
 * @pure
 */
function msgfmt_parse_message(string $locale, string $pattern, string $message): array|false
{
}

/**
 * @return bool
 */
function msgfmt_set_pattern(MessageFormatter $formatter, string $pattern): bool
{
}

/**
 * @pure
 */
function msgfmt_get_pattern(MessageFormatter $formatter): string|false
{
}

/**
 * @pure
 */
function msgfmt_get_locale(MessageFormatter $formatter): string
{
}

function msgfmt_get_error_code(MessageFormatter $formatter): int
{
}

function msgfmt_get_error_message(MessageFormatter $formatter): string
{
}

/**
 * @param string|null $timezone
 *
 * @pure
 */
function datefmt_create(
    null|string $locale,
    int $dateType = 0,
    int $timeType = 0,
    $timezone = null,
    IntlCalendar|int|null $calendar = null,
    string|null $pattern = null,
): null|IntlDateFormatter {
}

/**
 * @pure
 */
function datefmt_get_datetype(IntlDateFormatter $formatter): int|false
{
}

/**
 * @pure
 */
function datefmt_get_timetype(IntlDateFormatter $formatter): int|false
{
}

/**
 * @pure
 */
function datefmt_get_calendar(IntlDateFormatter $formatter): int|false
{
}

function datefmt_set_calendar(IntlDateFormatter $formatter, IntlCalendar|int|null $calendar): bool
{
}

/**
 * @pure
 */
function datefmt_get_locale(IntlDateFormatter $formatter, int $type = ULOC_ACTUAL_LOCALE): string|false
{
}

/**
 * @pure
 */
function datefmt_get_timezone_id(IntlDateFormatter $formatter): string|false
{
}

/**
 * @pure
 */
function datefmt_get_calendar_object(IntlDateFormatter $formatter): IntlCalendar|false|null
{
}

/**
 * @pure
 */
function datefmt_get_timezone(IntlDateFormatter $formatter): IntlTimeZone|false
{
}

/**
 * @param IntlTimeZone|DateTimeZone|string|null $timezone
 */
function datefmt_set_timezone(IntlDateFormatter $formatter, $timezone): null|bool
{
}

/**
 * @pure
 */
function datefmt_get_pattern(IntlDateFormatter $formatter): string|false
{
}

function datefmt_set_pattern(IntlDateFormatter $formatter, string $pattern): bool
{
}

/**
 * @pure
 */
function datefmt_is_lenient(IntlDateFormatter $formatter): bool
{
}

function datefmt_set_lenient(IntlDateFormatter $formatter, bool $lenient): void
{
}

/**
 * @param object|array|string|int|float $datetime
 *
 * @pure
 */
function datefmt_format(IntlDateFormatter $formatter, $datetime): string|false
{
}

/**
 * @param IntlCalendar|DateTimeInterface $datetime
 * @param array|int|string|null $format
 *
 * @pure
 */
function datefmt_format_object($datetime, $format = null, null|string $locale = null): string|false
{
}

/**
 * @param-out int $offset
 */
function datefmt_parse(IntlDateFormatter $formatter, string $string, &$offset = null): int|float|false
{
}

function datefmt_localtime(IntlDateFormatter $formatter, string $string, &$offset = null): array|false
{
}

function datefmt_get_error_code(IntlDateFormatter $formatter): int
{
}

function datefmt_get_error_message(IntlDateFormatter $formatter): string
{
}

/**
 * @return int<0, max>|false|null
 *
 * @pure
 */
function grapheme_strlen(string $string): int|false|null
{
}

/**
 * @return int<0, max>|false
 *
 * @pure
 */
function grapheme_strpos(string $haystack, string $needle, int $offset = 0): int|false
{
}

/**
 * @return int<0, max>|false
 *
 * @pure
 */
function grapheme_stripos(string $haystack, string $needle, int $offset = 0): int|false
{
}

/**
 * @return int<0, max>|false
 *
 * @pure
 */
function grapheme_strrpos(string $haystack, string $needle, int $offset = 0): int|false
{
}

/**
 * @return int<0, max>|false
 *
 * @pure
 */
function grapheme_strripos(string $haystack, string $needle, int $offset = 0): int|false
{
}

/**
 * @pure
 */
function grapheme_substr(string $string, int $offset, null|int $length = null): string|false
{
}

/**
 * @pure
 */
function grapheme_strstr(string $haystack, string $needle, bool $beforeNeedle = false): string|false
{
}

/**
 * @pure
 */
function grapheme_stristr(string $haystack, string $needle, bool $beforeNeedle = false): string|false
{
}

/**
 * @param-out int $next
 */
function grapheme_extract(string $haystack, int $size, int $type = 0, int $offset = 0, &$next = null): string|false
{
}

/**
 * @param-out array $idna_info
 */
function idn_to_ascii(
    string $domain,
    int $flags = 0,
    int $variant = INTL_IDNA_VARIANT_UTS46,
    &$idna_info = null,
): string|false {
}

/**
 * @param-out array $idna_info
 */
function idn_to_utf8(
    string $domain,
    int $flags = 0,
    int $variant = INTL_IDNA_VARIANT_UTS46,
    &$idna_info = null,
): string|false {
}

/**
 * @pure
 */
function intlcal_create_instance(
    IntlTimeZone|DateTimeZone|string|null $timezone = null,
    null|string $locale = null,
): null|IntlCalendar {
}

/**
 * @pure
 */
function intlcal_get_keyword_values_for_locale(string $keyword, string $locale, bool $onlyCommon): IntlIterator|false
{
}

function intlcal_get_now(): float
{
}

/**
 * @return string[]
 *
 * @pure
 */
function intlcal_get_available_locales(): array
{
}

/**
 * @pure
 */
function intl_get(IntlCalendar $calendar, int $field): int
{
}

/**
 * @pure
 */
function intlcal_get_time(IntlCalendar $calendar): float|false
{
}

function intlcal_set_time(IntlCalendar $calendar, float $timestamp): bool
{
}

function intlcal_add(IntlCalendar $calendar, int $field, int $value): bool
{
}

function intlcal_set_time_zone(IntlCalendar $calendar, $timezone): bool
{
}

/**
 * @pure
 */
function intlcal_after(IntlCalendar $calendar, IntlCalendar $other): bool
{
}

/**
 * @pure
 */
function intlcal_before(IntlCalendar $calendar, IntlCalendar $other): bool
{
}

#[Deprecated(
    reason: 'use IntlCalendar::set(), IntlCalendar::setDate(), or IntlCalendar::setDateTime() instead',
    since: '8.4',
)]
function intlcal_set(
    IntlCalendar $calendar,
    int $year,
    int $month,
    int $dayOfMonth,
    int $hour,
    int $minute,
    int $second,
): bool {
}

function intlcal_roll(IntlCalendar $calendar, int $field, int|bool $value): bool
{
}

function intlcal_clear(IntlCalendar $calendar, null|int $field = null): bool
{
}

/**
 * @pure
 */
function intlcal_field_difference(IntlCalendar $calendar, float $timestamp, int $field): int|false
{
}

/**
 * @pure
 */
function intlcal_get_actual_maximum(IntlCalendar $calendar, int $field): int|false
{
}

/**
 * @pure
 */
function intlcal_get_actual_minimum(IntlCalendar $calendar, int $field): int|false
{
}

/**
 * @pure
 */
function intlcal_get_day_of_week_type(IntlCalendar $calendar, int $dayOfWeek): int|false
{
}

/**
 * @pure
 */
function intlcal_get_first_day_of_week(IntlCalendar $calendar): int|false
{
}

/**
 * @pure
 */
function intlcal_greates_minimum($calendar, $field)
{
}

/**
 * @pure
 */
function intlcal_get(IntlCalendar $calendar, int $field): int|false
{
}

/**
 * @pure
 */
function intlcal_get_least_maximum(IntlCalendar $calendar, int $field): int|false
{
}

/**
 * @pure
 */
function intlcal_get_greatest_minimum(IntlCalendar $calendar, int $field): int|false
{
}

/**
 * @pure
 */
function intlcal_get_locale(IntlCalendar $calendar, int $type): string|false
{
}

/**
 * @pure
 */
function intcal_get_maximum(IntlCalendar $calendar, int $field): int|false
{
}

/**
 * @pure
 */
function intlcal_get_minimal_days_in_first_week(IntlCalendar $calendar): int|false
{
}

/**
 * @pure
 */
function intlcal_get_minimum(IntlCalendar $calendar, int $field): int|false
{
}

/**
 * @pure
 */
function intlcal_get_time_zone(IntlCalendar $calendar): IntlTimeZone|false
{
}

/**
 * @pure
 */
function intlcal_get_type(IntlCalendar $calendar): string
{
}

/**
 * @pure
 */
function intlcal_get_weekend_transition(IntlCalendar $calendar, int $dayOfWeek): int|false
{
}

/**
 * @pure
 */
function intlcal_in_daylight_time(IntlCalendar $calendar): bool
{
}

/**
 * @pure
 */
function intlcal_is_lenient(IntlCalendar $calendar): bool
{
}

/**
 * @pure
 */
function intlcal_is_set(IntlCalendar $calendar, int $field): bool
{
}

/**
 * @pure
 */
function intlcal_get_maximum(IntlCalendar $calendar, int $field): false|int
{
}

/**
 * @pure
 */
function intlcal_is_equivalent_to(IntlCalendar $calendar, IntlCalendar $other): bool
{
}

/**
 * @pure
 */
function intlcal_is_weekend(IntlCalendar $calendar, null|float $timestamp = null): bool
{
}

function intlcal_set_first_day_of_week(IntlCalendar $calendar, int $dayOfWeek): bool
{
}

function intlcal_set_lenient(IntlCalendar $calendar, bool $lenient): bool
{
}

/**
 * @pure
 */
function intlcal_get_repeated_wall_time_option(IntlCalendar $calendar): int
{
}

/**
 * @pure
 */
function intlcal_equals(IntlCalendar $calendar, IntlCalendar $other): bool
{
}

/**
 * @pure
 */
function intlcal_get_skipped_wall_time_option(IntlCalendar $calendar): int
{
}

function intlcal_set_repeated_wall_time_option(IntlCalendar $calendar, int $option): bool
{
}

function intlcal_set_skipped_wall_time_option(IntlCalendar $calendar, int $option): bool
{
}

/**
 * @pure
 */
function intlcal_from_date_time(DateTime|string $datetime, null|string $locale = null): null|IntlCalendar
{
}

/**
 * @pure
 */
function intlcal_to_date_time(IntlCalendar $calendar): DateTime|false
{
}

function intlcal_get_error_code(IntlCalendar $calendar): int|false
{
}

function intlcal_get_error_message(IntlCalendar $calendar): string|false
{
}

/**
 * @pure
 */
function intltz_count_equivalent_ids(string $timezoneId): int|false
{
}

/**
 * @return IntlTimeZone
 *
 * @pure
 */
function intlz_create_default()
{
}

/**
 * @param IntlTimeZone|string|int|float|null $countryOrRawOffset
 *
 * @pure
 */
function intltz_create_enumeration($countryOrRawOffset): IntlIterator|false
{
}

/**
 * @pure
 */
function intltz_create_time_zone(string $timezoneId): null|IntlTimeZone
{
}

/**
 * @pure
 */
function intltz_from_date_time_zone(DateTimeZone $timezone): null|IntlTimeZone
{
}

/**
 * @param-out bool $isSystemId
 *
 * @pure
 */
function intltz_get_canonical_id(string $timezoneId, &$isSystemId): string|false
{
}

/**
 * @pure
 */
function intltz_get_display_name(
    IntlTimeZone $timezone,
    bool $dst = false,
    int $style = 2,
    null|string $locale,
): string|false {
}

/**
 * @pure
 */
function intltz_get_dst_savings(IntlTimeZone $timezone): int
{
}

/**
 * @pure
 */
function intltz_get_equivalent_id(string $timezoneId, int $offset): string|false
{
}

function intltz_get_error_code(IntlTimeZone $timezone): int|false
{
}

function intltz_get_error_message(IntlTimeZone $timezone): string|false
{
}

/**
 * @pure
 */
function intltz_getGMT(): IntlTimeZone
{
}

/**
 * @pure
 */
function intltz_get_id(IntlTimeZone $timezone): string|false
{
}

/**
 * @param-out int $rawOffset
 * @param-out int $dstOffset
 *
 * @pure
 */
function intltz_get_offset(IntlTimeZone $timezone, float $timestamp, bool $local, &$rawOffset, &$dstOffset): bool
{
}

/**
 * @pure
 */
function intltz_get_raw_offset(IntlTimeZone $timezone): int
{
}

/**
 * @pure
 */
function intltz_get_tz_data_version(): string|false
{
}

/**
 * @pure
 */
function intltz_has_same_rules(IntlTimeZone $timezone, IntlTimeZone $other): bool
{
}

/**
 * @pure
 */
function intltz_to_date_time_zone(IntlTimeZone $timezone): DateTimeZone|false
{
}

/**
 * @pure
 */
function intltz_use_daylight_time(IntlTimeZone $timezone): bool
{
}

/**
 * @param DateTimeZone|IntlTimeZone|string|int|null $timezoneOrYear
 * @param string|null $localeOrMonth
 * @param int $day
 * @param int $hour
 * @param int $minute
 * @param int $second
 *
 * @pure
 */
#[Deprecated(
    'use IntlGregorianCalendar::__construct(), IntlGregorianCalendar::createFromDate(), or IntlGregorianCalendar::createFromDateTime() instead',
)]
function intlgregcal_create_instance(
    $timezoneOrYear,
    $localeOrMonth,
    $day,
    $hour,
    $minute,
    $second,
): null|IntlGregorianCalendar {
}

function intlgregcal_set_gregorian_change(IntlGregorianCalendar $calendar, float $timestamp): bool
{
}

/**
 * @pure
 */
function intlgregcal_get_gregorian_change(IntlGregorianCalendar $calendar): float
{
}

/**
 * @pure
 */
function intlgregcal_is_leap_year(IntlGregorianCalendar $calendar, int $year): bool
{
}

function resourcebundle_create(null|string $locale, null|string $bundle, bool $fallback = true): null|ResourceBundle
{
}

/**
 * @pure
 */
function resourcebundle_get(ResourceBundle $bundle, string|int $index, bool $fallback = true)
{
}

/**
 * @pure
 */
function resourcebundle_count(ResourceBundle $bundle): int
{
}

/**
 * @pure
 */
function resourcebundle_locales(string $bundle): array|false
{
}

function resourcebundle_get_error_code(ResourceBundle $bundle): int
{
}

function resourcebundle_get_error_message(ResourceBundle $bundle): string
{
}

/**
 * @pure
 */
function transliterator_create(string $id, int $direction = 0): null|Transliterator
{
}

/**
 * @pure
 */
function transliterator_create_from_rules(string $rules, int $direction = 0): null|Transliterator
{
}

/**
 * @return string[]|false
 *
 * @pure
 */
function transliterator_list_ids(): array|false
{
}

/**
 * @pure
 */
function transliterator_create_inverse(Transliterator $transliterator): null|Transliterator
{
}

/**
 * @pure
 */
function transliterator_transliterate(
    Transliterator|string $transliterator,
    string $string,
    int $start = 0,
    int $end = -1,
): string|false {
}

function transliterator_get_error_code(Transliterator $transliterator): int|false
{
}

function transliterator_get_error_message(Transliterator $transliterator): string|false
{
}

function intl_get_error_code(): int
{
}

function intl_get_error_message(): string
{
}

/**
 * @pure
 */
function intl_is_failure(int $errorCode): bool
{
}

function intl_error_name(int $errorCode): string
{
}

/**
 * @pure
 */
function normalizer_get_raw_decomposition(string $string, int $form = Normalizer::FORM_C): null|string
{
}

/**
 * @pure
 */
function intltz_create_default(): IntlTimeZone
{
}

/**
 * @pure
 */
function intltz_get_gmt(): IntlTimeZone
{
}

/**
 * @pure
 */
function intltz_get_unknown(): IntlTimeZone
{
}

/**
 * @pure
 */
function intltz_create_time_zone_id_enumeration(
    int $type,
    null|string $region = null,
    null|int $rawOffset = null,
): IntlIterator|false {
}

/**
 * @pure
 */
function intltz_get_region(string $timezoneId): string|false
{
}

function intlcal_set_minimal_days_in_first_week(IntlCalendar $calendar, int $days): bool
{
}

function intltz_get_windows_id(string $timezoneId): string|false
{
}

function intltz_get_id_for_windows_id(string $timezoneId, null|string $region = null): string|false
{
}

function grapheme_str_split(string $string, int $length = 1): array|false
{
}

function intltz_get_iana_id(string $timezoneId): string|false
{
}

const INTL_MAX_LOCALE_LEN = 156;

const INTL_ICU_VERSION = '74.1';

const INTL_ICU_DATA_VERSION = '74.1';

const ULOC_ACTUAL_LOCALE = 0;

const ULOC_VALID_LOCALE = 1;

const GRAPHEME_EXTR_COUNT = 0;

const GRAPHEME_EXTR_MAXBYTES = 1;

const GRAPHEME_EXTR_MAXCHARS = 2;

const U_USING_FALLBACK_WARNING = -128;

const U_ERROR_WARNING_START = -128;

const U_USING_DEFAULT_WARNING = -127;

const U_SAFECLONE_ALLOCATED_WARNING = -126;

const U_STATE_OLD_WARNING = -125;

const U_STRING_NOT_TERMINATED_WARNING = -124;

const U_SORT_KEY_TOO_SHORT_WARNING = -123;

const U_AMBIGUOUS_ALIAS_WARNING = -122;

const U_DIFFERENT_UCA_VERSION = -121;

const U_ERROR_WARNING_LIMIT = -119;

const U_ZERO_ERROR = 0;

const U_ILLEGAL_ARGUMENT_ERROR = 1;

const U_MISSING_RESOURCE_ERROR = 2;

const U_INVALID_FORMAT_ERROR = 3;

const U_FILE_ACCESS_ERROR = 4;

const U_INTERNAL_PROGRAM_ERROR = 5;

const U_MESSAGE_PARSE_ERROR = 6;

const U_MEMORY_ALLOCATION_ERROR = 7;

const U_INDEX_OUTOFBOUNDS_ERROR = 8;

const U_PARSE_ERROR = 9;

const U_INVALID_CHAR_FOUND = 10;

const U_TRUNCATED_CHAR_FOUND = 11;

const U_ILLEGAL_CHAR_FOUND = 12;

const U_INVALID_TABLE_FORMAT = 13;

const U_INVALID_TABLE_FILE = 14;

const U_BUFFER_OVERFLOW_ERROR = 15;

const U_UNSUPPORTED_ERROR = 16;

const U_RESOURCE_TYPE_MISMATCH = 17;

const U_ILLEGAL_ESCAPE_SEQUENCE = 18;

const U_UNSUPPORTED_ESCAPE_SEQUENCE = 19;

const U_NO_SPACE_AVAILABLE = 20;

const U_CE_NOT_FOUND_ERROR = 21;

const U_PRIMARY_TOO_LONG_ERROR = 22;

const U_STATE_TOO_OLD_ERROR = 23;

const U_TOO_MANY_ALIASES_ERROR = 24;

const U_ENUM_OUT_OF_SYNC_ERROR = 25;

const U_INVARIANT_CONVERSION_ERROR = 26;

const U_INVALID_STATE_ERROR = 27;

const U_COLLATOR_VERSION_MISMATCH = 28;

const U_USELESS_COLLATOR_ERROR = 29;

const U_NO_WRITE_PERMISSION = 30;

const U_STANDARD_ERROR_LIMIT = 32;

const U_BAD_VARIABLE_DEFINITION = 65536;

const U_PARSE_ERROR_START = 65536;

const U_MALFORMED_RULE = 65537;

const U_MALFORMED_SET = 65538;

const U_MALFORMED_SYMBOL_REFERENCE = 65539;

const U_MALFORMED_UNICODE_ESCAPE = 65540;

const U_MALFORMED_VARIABLE_DEFINITION = 65541;

const U_MALFORMED_VARIABLE_REFERENCE = 65542;

const U_MISMATCHED_SEGMENT_DELIMITERS = 65543;

const U_MISPLACED_ANCHOR_START = 65544;

const U_MISPLACED_CURSOR_OFFSET = 65545;

const U_MISPLACED_QUANTIFIER = 65546;

const U_MISSING_OPERATOR = 65547;

const U_MISSING_SEGMENT_CLOSE = 65548;

const U_MULTIPLE_ANTE_CONTEXTS = 65549;

const U_MULTIPLE_CURSORS = 65550;

const U_MULTIPLE_POST_CONTEXTS = 65551;

const U_TRAILING_BACKSLASH = 65552;

const U_UNDEFINED_SEGMENT_REFERENCE = 65553;

const U_UNDEFINED_VARIABLE = 65554;

const U_UNQUOTED_SPECIAL = 65555;

const U_UNTERMINATED_QUOTE = 65556;

const U_RULE_MASK_ERROR = 65557;

const U_MISPLACED_COMPOUND_FILTER = 65558;

const U_MULTIPLE_COMPOUND_FILTERS = 65559;

const U_INVALID_RBT_SYNTAX = 65560;

const U_INVALID_PROPERTY_PATTERN = 65561;

const U_MALFORMED_PRAGMA = 65562;

const U_UNCLOSED_SEGMENT = 65563;

const U_ILLEGAL_CHAR_IN_SEGMENT = 65564;

const U_VARIABLE_RANGE_EXHAUSTED = 65565;

const U_VARIABLE_RANGE_OVERLAP = 65566;

const U_ILLEGAL_CHARACTER = 65567;

const U_INTERNAL_TRANSLITERATOR_ERROR = 65568;

const U_INVALID_ID = 65569;

const U_INVALID_FUNCTION = 65570;

const U_PARSE_ERROR_LIMIT = 65571;

const U_UNEXPECTED_TOKEN = 65792;

const U_FMT_PARSE_ERROR_START = 65792;

const U_MULTIPLE_DECIMAL_SEPARATORS = 65793;

const U_MULTIPLE_DECIMAL_SEPERATORS = 65793;

const U_MULTIPLE_EXPONENTIAL_SYMBOLS = 65794;

const U_MALFORMED_EXPONENTIAL_PATTERN = 65795;

const U_MULTIPLE_PERCENT_SYMBOLS = 65796;

const U_MULTIPLE_PERMILL_SYMBOLS = 65797;

const U_MULTIPLE_PAD_SPECIFIERS = 65798;

const U_PATTERN_SYNTAX_ERROR = 65799;

const U_ILLEGAL_PAD_POSITION = 65800;

const U_UNMATCHED_BRACES = 65801;

const U_UNSUPPORTED_PROPERTY = 65802;

const U_UNSUPPORTED_ATTRIBUTE = 65803;

const U_FMT_PARSE_ERROR_LIMIT = 65812;

const U_BRK_INTERNAL_ERROR = 66048;

const U_BRK_ERROR_START = 66048;

const U_BRK_HEX_DIGITS_EXPECTED = 66049;

const U_BRK_SEMICOLON_EXPECTED = 66050;

const U_BRK_RULE_SYNTAX = 66051;

const U_BRK_UNCLOSED_SET = 66052;

const U_BRK_ASSIGN_ERROR = 66053;

const U_BRK_VARIABLE_REDFINITION = 66054;

const U_BRK_MISMATCHED_PAREN = 66055;

const U_BRK_NEW_LINE_IN_QUOTED_STRING = 66056;

const U_BRK_UNDEFINED_VARIABLE = 66057;

const U_BRK_INIT_ERROR = 66058;

const U_BRK_RULE_EMPTY_SET = 66059;

const U_BRK_UNRECOGNIZED_OPTION = 66060;

const U_BRK_MALFORMED_RULE_TAG = 66061;

const U_BRK_ERROR_LIMIT = 66062;

const U_REGEX_INTERNAL_ERROR = 66304;

const U_REGEX_ERROR_START = 66304;

const U_REGEX_RULE_SYNTAX = 66305;

const U_REGEX_INVALID_STATE = 66306;

const U_REGEX_BAD_ESCAPE_SEQUENCE = 66307;

const U_REGEX_PROPERTY_SYNTAX = 66308;

const U_REGEX_UNIMPLEMENTED = 66309;

const U_REGEX_MISMATCHED_PAREN = 66310;

const U_REGEX_NUMBER_TOO_BIG = 66311;

const U_REGEX_BAD_INTERVAL = 66312;

const U_REGEX_MAX_LT_MIN = 66313;

const U_REGEX_INVALID_BACK_REF = 66314;

const U_REGEX_INVALID_FLAG = 66315;

const U_REGEX_LOOK_BEHIND_LIMIT = 66316;

const U_REGEX_SET_CONTAINS_STRING = 66317;

const U_REGEX_ERROR_LIMIT = 66326;

const U_IDNA_PROHIBITED_ERROR = 66560;

const U_IDNA_ERROR_START = 66560;

const U_IDNA_UNASSIGNED_ERROR = 66561;

const U_IDNA_CHECK_BIDI_ERROR = 66562;

const U_IDNA_STD3_ASCII_RULES_ERROR = 66563;

const U_IDNA_ACE_PREFIX_ERROR = 66564;

const U_IDNA_VERIFICATION_ERROR = 66565;

const U_IDNA_LABEL_TOO_LONG_ERROR = 66566;

const U_IDNA_ZERO_LENGTH_LABEL_ERROR = 66567;

const U_IDNA_DOMAIN_NAME_TOO_LONG_ERROR = 66568;

const U_IDNA_ERROR_LIMIT = 66569;

const U_STRINGPREP_PROHIBITED_ERROR = 66560;

const U_STRINGPREP_UNASSIGNED_ERROR = 66561;

const U_STRINGPREP_CHECK_BIDI_ERROR = 66562;

const U_ERROR_LIMIT = 66818;

const IDNA_DEFAULT = 0;

const IDNA_ALLOW_UNASSIGNED = 1;

const IDNA_USE_STD3_RULES = 2;

const IDNA_CHECK_BIDI = 4;

const IDNA_CHECK_CONTEXTJ = 8;

const IDNA_NONTRANSITIONAL_TO_ASCII = 16;

const IDNA_NONTRANSITIONAL_TO_UNICODE = 32;

const INTL_IDNA_VARIANT_2003 = 0;

const INTL_IDNA_VARIANT_UTS46 = 1;

const IDNA_ERROR_EMPTY_LABEL = 1;

const IDNA_ERROR_LABEL_TOO_LONG = 2;

const IDNA_ERROR_DOMAIN_NAME_TOO_LONG = 4;

const IDNA_ERROR_LEADING_HYPHEN = 8;

const IDNA_ERROR_HYPHEN_3_4 = 32;

const IDNA_ERROR_LEADING_COMBINING_MARK = 64;

const IDNA_ERROR_DISALLOWED = 128;

const IDNA_ERROR_PUNYCODE = 256;

const IDNA_ERROR_LABEL_HAS_DOT = 512;

const IDNA_ERROR_INVALID_ACE_LABEL = 1024;

const IDNA_ERROR_BIDI = 2048;

const IDNA_ERROR_CONTEXTJ = 4096;

class IntlBreakIterator implements IteratorAggregate
{
    public const DONE = -1;
    public const WORD_NONE = 0;
    public const WORD_NONE_LIMIT = 100;
    public const WORD_NUMBER = 100;
    public const WORD_NUMBER_LIMIT = 200;
    public const WORD_LETTER = 200;
    public const WORD_LETTER_LIMIT = 300;
    public const WORD_KANA = 300;
    public const WORD_KANA_LIMIT = 400;
    public const WORD_IDEO = 400;
    public const WORD_IDEO_LIMIT = 500;
    public const LINE_SOFT = 0;
    public const LINE_SOFT_LIMIT = 100;
    public const LINE_HARD = 100;
    public const LINE_HARD_LIMIT = 200;
    public const SENTENCE_TERM = 0;
    public const SENTENCE_TERM_LIMIT = 100;
    public const SENTENCE_SEP = 100;
    public const SENTENCE_SEP_LIMIT = 200;

    private function __construct() {}

    public static function createCharacterInstance(string|null $locale = null): null|IntlBreakIterator
    {
    }

    public static function createCodePointInstance(): IntlCodePointBreakIterator
    {
    }

    public static function createLineInstance(string|null $locale): null|IntlBreakIterator
    {
    }

    public static function createSentenceInstance(string|null $locale): null|IntlBreakIterator
    {
    }

    public static function createTitleInstance(string|null $locale): null|IntlBreakIterator
    {
    }

    public static function createWordInstance(string|null $locale): null|IntlBreakIterator
    {
    }

    /**
     * @pure
     */
    public function current(): int
    {
    }

    public function first(): int
    {
    }

    public function following(int $offset): int
    {
    }

    /**
     * @pure
     */
    public function getErrorCode(): int
    {
    }

    /**
     * @pure
     */
    public function getErrorMessage(): string
    {
    }

    /**
     * @pure
     */
    public function getLocale(int $type): string|false
    {
    }

    /**
     * @param int $type
     *
     * @pure
     */
    public function getPartsIterator($type = IntlPartsIterator::KEY_SEQUENTIAL): IntlPartsIterator
    {
    }

    /**
     * @pure
     */
    public function getText(): null|string
    {
    }

    /**
     * @pure
     */
    public function isBoundary(int $offset): bool
    {
    }

    public function last(): int
    {
    }

    public function next(null|int $offset = null): int
    {
    }

    public function preceding(int $offset): int
    {
    }

    public function previous(): int
    {
    }

    public function setText(string $text): null|bool
    {
    }

    /**
     * @pure
     */
    public function getIterator(): Iterator
    {
    }
}

class IntlRuleBasedBreakIterator extends IntlBreakIterator implements Traversable
{
    /**
     * @param string $rules
     * @param string $compiled
     *
     * @pure
     */
    public function __construct(string $rules, bool $compiled = false) {}

    /**
     * @param string $locale
     *
     * @return IntlRuleBasedBreakIterator
     */
    public static function createCharacterInstance($locale)
    {
    }

    /**
     * @return IntlRuleBasedBreakIterator
     */
    public static function createCodePointInstance()
    {
    }

    /**
     * @param string $locale
     *
     * @return IntlRuleBasedBreakIterator
     */
    public static function createLineInstance($locale)
    {
    }

    /**
     * @param string $locale
     *
     * @return IntlRuleBasedBreakIterator
     */
    public static function createSentenceInstance($locale)
    {
    }

    /**
     * @param string $locale
     *
     * @return IntlRuleBasedBreakIterator
     */
    public static function createTitleInstance($locale)
    {
    }

    /**
     * @param string $locale
     *
     * @return IntlRuleBasedBreakIterator
     */
    public static function createWordInstance($locale)
    {
    }

    /**
     * @pure
     */
    public function getBinaryRules(): string|false
    {
    }

    /**
     * @pure
     */
    public function getRules(): string|false
    {
    }

    /**
     * @pure
     */
    public function getRuleStatus(): int
    {
    }

    /**
     * @pure
     */
    public function getRuleStatusVec(): array|false
    {
    }
}

class IntlPartsIterator extends IntlIterator implements Iterator
{
    public const KEY_SEQUENTIAL = 0;
    public const KEY_LEFT = 1;
    public const KEY_RIGHT = 2;

    /**
     * @pure
     */
    public function getBreakIterator(): IntlBreakIterator
    {
    }

    public function getRuleStatus(): int
    {
    }
}

class IntlCodePointBreakIterator extends IntlBreakIterator implements Traversable
{
    /**
     * @pure
     */
    public function getLastCodePoint(): int
    {
    }
}

class UConverter
{
    public const REASON_UNASSIGNED = 0;
    public const REASON_ILLEGAL = 1;
    public const REASON_IRREGULAR = 2;
    public const REASON_RESET = 3;
    public const REASON_CLOSE = 4;
    public const REASON_CLONE = 5;
    public const UNSUPPORTED_CONVERTER = -1;
    public const SBCS = 0;
    public const DBCS = 1;
    public const MBCS = 2;
    public const LATIN_1 = 3;
    public const UTF8 = 4;
    public const UTF16_BigEndian = 5;
    public const UTF16_LittleEndian = 6;
    public const UTF32_BigEndian = 7;
    public const UTF32_LittleEndian = 8;
    public const EBCDIC_STATEFUL = 9;
    public const ISO_2022 = 10;
    public const LMBCS_1 = 11;
    public const LMBCS_2 = 12;
    public const LMBCS_3 = 13;
    public const LMBCS_4 = 14;
    public const LMBCS_5 = 15;
    public const LMBCS_6 = 16;
    public const LMBCS_8 = 17;
    public const LMBCS_11 = 18;
    public const LMBCS_16 = 19;
    public const LMBCS_17 = 20;
    public const LMBCS_18 = 21;
    public const LMBCS_19 = 22;
    public const LMBCS_LAST = 22;
    public const HZ = 23;
    public const SCSU = 24;
    public const ISCII = 25;
    public const US_ASCII = 26;
    public const UTF7 = 27;
    public const BOCU1 = 28;
    public const UTF16 = 29;
    public const UTF32 = 30;
    public const CESU8 = 31;
    public const IMAP_MAILBOX = 32;

    /**
     * @pure
     */
    public function __construct(string|null $destination_encoding = null, string|null $source_encoding = null) {}

    /**
     * @pure
     */
    public function convert(string $str, bool $reverse = false): string|false
    {
    }

    /**
     * @param-out int $error
     */
    public function fromUCallback(int $reason, array $source, int $codePoint, &$error): array|string|int|null
    {
    }

    public static function getAliases(string $name): array|false|null
    {
    }

    public static function getAvailable(): array
    {
    }

    /**
     * @pure
     */
    public function getDestinationEncoding(): string|false|null
    {
    }

    /**
     * @pure
     */
    public function getDestinationType(): int|false|null
    {
    }

    /**
     * @pure
     */
    public function getErrorCode(): int
    {
    }

    /**
     * @pure
     */
    public function getErrorMessage(): null|string
    {
    }

    /**
     * @pure
     */
    public function getSourceEncoding(): string|false|null
    {
    }

    /**
     * @pure
     */
    public function getSourceType(): int|false|null
    {
    }

    /**
     * @pure
     */
    public static function getStandards(): null|array
    {
    }

    /**
     * @pure
     */
    public function getSubstChars(): string|false|null
    {
    }

    /**
     * @pure
     */
    public static function reasonText(int $reason): string
    {
    }

    public function setDestinationEncoding(string $encoding): bool
    {
    }

    public function setSourceEncoding(string $encoding): bool
    {
    }

    public function setSubstChars(string $chars): bool
    {
    }

    /**
     * @param-out int $error
     */
    public function toUCallback(int $reason, string $source, string $codeUnits, &$error): array|string|int|null
    {
    }

    public static function transcode(
        string $str,
        string $toEncoding,
        string $fromEncoding,
        array|null $options = null,
    ): string|false {
    }
}

class IntlChar
{
    public const UNICODE_VERSION = 13.0;
    public const CODEPOINT_MIN = 0;
    public const CODEPOINT_MAX = 1114111;
    public const FOLD_CASE_DEFAULT = 0;
    public const FOLD_CASE_EXCLUDE_SPECIAL_I = 1;
    public const PROPERTY_ALPHABETIC = 0;
    public const PROPERTY_BINARY_START = 0;
    public const PROPERTY_ASCII_HEX_DIGIT = 1;
    public const PROPERTY_BIDI_CONTROL = 2;
    public const PROPERTY_BIDI_MIRRORED = 3;
    public const PROPERTY_DASH = 4;
    public const PROPERTY_DEFAULT_IGNORABLE_CODE_POINT = 5;
    public const PROPERTY_DEPRECATED = 6;
    public const PROPERTY_DIACRITIC = 7;
    public const PROPERTY_EXTENDER = 8;
    public const PROPERTY_FULL_COMPOSITION_EXCLUSION = 9;
    public const PROPERTY_GRAPHEME_BASE = 10;
    public const PROPERTY_GRAPHEME_EXTEND = 11;
    public const PROPERTY_GRAPHEME_LINK = 12;
    public const PROPERTY_HEX_DIGIT = 13;
    public const PROPERTY_HYPHEN = 14;
    public const PROPERTY_ID_CONTINUE = 15;
    public const PROPERTY_ID_START = 16;
    public const PROPERTY_IDEOGRAPHIC = 17;
    public const PROPERTY_IDS_BINARY_OPERATOR = 18;
    public const PROPERTY_IDS_TRINARY_OPERATOR = 19;
    public const PROPERTY_JOIN_CONTROL = 20;
    public const PROPERTY_LOGICAL_ORDER_EXCEPTION = 21;
    public const PROPERTY_LOWERCASE = 22;
    public const PROPERTY_MATH = 23;
    public const PROPERTY_NONCHARACTER_CODE_POINT = 24;
    public const PROPERTY_QUOTATION_MARK = 25;
    public const PROPERTY_RADICAL = 26;
    public const PROPERTY_SOFT_DOTTED = 27;
    public const PROPERTY_TERMINAL_PUNCTUATION = 28;
    public const PROPERTY_UNIFIED_IDEOGRAPH = 29;
    public const PROPERTY_UPPERCASE = 30;
    public const PROPERTY_WHITE_SPACE = 31;
    public const PROPERTY_XID_CONTINUE = 32;
    public const PROPERTY_XID_START = 33;
    public const PROPERTY_CASE_SENSITIVE = 34;
    public const PROPERTY_S_TERM = 35;
    public const PROPERTY_VARIATION_SELECTOR = 36;
    public const PROPERTY_NFD_INERT = 37;
    public const PROPERTY_NFKD_INERT = 38;
    public const PROPERTY_NFC_INERT = 39;
    public const PROPERTY_NFKC_INERT = 40;
    public const PROPERTY_SEGMENT_STARTER = 41;
    public const PROPERTY_PATTERN_SYNTAX = 42;
    public const PROPERTY_PATTERN_WHITE_SPACE = 43;
    public const PROPERTY_POSIX_ALNUM = 44;
    public const PROPERTY_POSIX_BLANK = 45;
    public const PROPERTY_POSIX_GRAPH = 46;
    public const PROPERTY_POSIX_PRINT = 47;
    public const PROPERTY_POSIX_XDIGIT = 48;
    public const PROPERTY_CASED = 49;
    public const PROPERTY_CASE_IGNORABLE = 50;
    public const PROPERTY_CHANGES_WHEN_LOWERCASED = 51;
    public const PROPERTY_CHANGES_WHEN_UPPERCASED = 52;
    public const PROPERTY_CHANGES_WHEN_TITLECASED = 53;
    public const PROPERTY_CHANGES_WHEN_CASEFOLDED = 54;
    public const PROPERTY_CHANGES_WHEN_CASEMAPPED = 55;
    public const PROPERTY_CHANGES_WHEN_NFKC_CASEFOLDED = 56;
    public const PROPERTY_BINARY_LIMIT = 65;
    public const PROPERTY_BIDI_CLASS = 4096;
    public const PROPERTY_INT_START = 4096;
    public const PROPERTY_BLOCK = 4097;
    public const PROPERTY_CANONICAL_COMBINING_CLASS = 4098;
    public const PROPERTY_DECOMPOSITION_TYPE = 4099;
    public const PROPERTY_EAST_ASIAN_WIDTH = 4100;
    public const PROPERTY_GENERAL_CATEGORY = 4101;
    public const PROPERTY_JOINING_GROUP = 4102;
    public const PROPERTY_JOINING_TYPE = 4103;
    public const PROPERTY_LINE_BREAK = 4104;
    public const PROPERTY_NUMERIC_TYPE = 4105;
    public const PROPERTY_SCRIPT = 4106;
    public const PROPERTY_HANGUL_SYLLABLE_TYPE = 4107;
    public const PROPERTY_NFD_QUICK_CHECK = 4108;
    public const PROPERTY_NFKD_QUICK_CHECK = 4109;
    public const PROPERTY_NFC_QUICK_CHECK = 4110;
    public const PROPERTY_NFKC_QUICK_CHECK = 4111;
    public const PROPERTY_LEAD_CANONICAL_COMBINING_CLASS = 4112;
    public const PROPERTY_TRAIL_CANONICAL_COMBINING_CLASS = 4113;
    public const PROPERTY_GRAPHEME_CLUSTER_BREAK = 4114;
    public const PROPERTY_SENTENCE_BREAK = 4115;
    public const PROPERTY_WORD_BREAK = 4116;
    public const PROPERTY_BIDI_PAIRED_BRACKET_TYPE = 4117;
    public const PROPERTY_INT_LIMIT = 4121;
    public const PROPERTY_GENERAL_CATEGORY_MASK = 8192;
    public const PROPERTY_MASK_START = 8192;
    public const PROPERTY_MASK_LIMIT = 8193;
    public const PROPERTY_NUMERIC_VALUE = 12288;
    public const PROPERTY_DOUBLE_START = 12288;
    public const PROPERTY_DOUBLE_LIMIT = 12289;
    public const PROPERTY_AGE = 16384;
    public const PROPERTY_STRING_START = 16384;
    public const PROPERTY_BIDI_MIRRORING_GLYPH = 16385;
    public const PROPERTY_CASE_FOLDING = 16386;
    public const PROPERTY_ISO_COMMENT = 16387;
    public const PROPERTY_LOWERCASE_MAPPING = 16388;
    public const PROPERTY_NAME = 16389;
    public const PROPERTY_SIMPLE_CASE_FOLDING = 16390;
    public const PROPERTY_SIMPLE_LOWERCASE_MAPPING = 16391;
    public const PROPERTY_SIMPLE_TITLECASE_MAPPING = 16392;
    public const PROPERTY_SIMPLE_UPPERCASE_MAPPING = 16393;
    public const PROPERTY_TITLECASE_MAPPING = 16394;
    public const PROPERTY_UNICODE_1_NAME = 16395;
    public const PROPERTY_UPPERCASE_MAPPING = 16396;
    public const PROPERTY_BIDI_PAIRED_BRACKET = 16397;
    public const PROPERTY_STRING_LIMIT = 16398;
    public const PROPERTY_SCRIPT_EXTENSIONS = 28672;
    public const PROPERTY_OTHER_PROPERTY_START = 28672;
    public const PROPERTY_OTHER_PROPERTY_LIMIT = 28673;
    public const PROPERTY_INVALID_CODE = -1;
    public const CHAR_CATEGORY_UNASSIGNED = 0;
    public const CHAR_CATEGORY_GENERAL_OTHER_TYPES = 0;
    public const CHAR_CATEGORY_UPPERCASE_LETTER = 1;
    public const CHAR_CATEGORY_LOWERCASE_LETTER = 2;
    public const CHAR_CATEGORY_TITLECASE_LETTER = 3;
    public const CHAR_CATEGORY_MODIFIER_LETTER = 4;
    public const CHAR_CATEGORY_OTHER_LETTER = 5;
    public const CHAR_CATEGORY_NON_SPACING_MARK = 6;
    public const CHAR_CATEGORY_ENCLOSING_MARK = 7;
    public const CHAR_CATEGORY_COMBINING_SPACING_MARK = 8;
    public const CHAR_CATEGORY_DECIMAL_DIGIT_NUMBER = 9;
    public const CHAR_CATEGORY_LETTER_NUMBER = 10;
    public const CHAR_CATEGORY_OTHER_NUMBER = 11;
    public const CHAR_CATEGORY_SPACE_SEPARATOR = 12;
    public const CHAR_CATEGORY_LINE_SEPARATOR = 13;
    public const CHAR_CATEGORY_PARAGRAPH_SEPARATOR = 14;
    public const CHAR_CATEGORY_CONTROL_CHAR = 15;
    public const CHAR_CATEGORY_FORMAT_CHAR = 16;
    public const CHAR_CATEGORY_PRIVATE_USE_CHAR = 17;
    public const CHAR_CATEGORY_SURROGATE = 18;
    public const CHAR_CATEGORY_DASH_PUNCTUATION = 19;
    public const CHAR_CATEGORY_START_PUNCTUATION = 20;
    public const CHAR_CATEGORY_END_PUNCTUATION = 21;
    public const CHAR_CATEGORY_CONNECTOR_PUNCTUATION = 22;
    public const CHAR_CATEGORY_OTHER_PUNCTUATION = 23;
    public const CHAR_CATEGORY_MATH_SYMBOL = 24;
    public const CHAR_CATEGORY_CURRENCY_SYMBOL = 25;
    public const CHAR_CATEGORY_MODIFIER_SYMBOL = 26;
    public const CHAR_CATEGORY_OTHER_SYMBOL = 27;
    public const CHAR_CATEGORY_INITIAL_PUNCTUATION = 28;
    public const CHAR_CATEGORY_FINAL_PUNCTUATION = 29;
    public const CHAR_CATEGORY_CHAR_CATEGORY_COUNT = 30;
    public const CHAR_DIRECTION_LEFT_TO_RIGHT = 0;
    public const CHAR_DIRECTION_RIGHT_TO_LEFT = 1;
    public const CHAR_DIRECTION_EUROPEAN_NUMBER = 2;
    public const CHAR_DIRECTION_EUROPEAN_NUMBER_SEPARATOR = 3;
    public const CHAR_DIRECTION_EUROPEAN_NUMBER_TERMINATOR = 4;
    public const CHAR_DIRECTION_ARABIC_NUMBER = 5;
    public const CHAR_DIRECTION_COMMON_NUMBER_SEPARATOR = 6;
    public const CHAR_DIRECTION_BLOCK_SEPARATOR = 7;
    public const CHAR_DIRECTION_SEGMENT_SEPARATOR = 8;
    public const CHAR_DIRECTION_WHITE_SPACE_NEUTRAL = 9;
    public const CHAR_DIRECTION_OTHER_NEUTRAL = 10;
    public const CHAR_DIRECTION_LEFT_TO_RIGHT_EMBEDDING = 11;
    public const CHAR_DIRECTION_LEFT_TO_RIGHT_OVERRIDE = 12;
    public const CHAR_DIRECTION_RIGHT_TO_LEFT_ARABIC = 13;
    public const CHAR_DIRECTION_RIGHT_TO_LEFT_EMBEDDING = 14;
    public const CHAR_DIRECTION_RIGHT_TO_LEFT_OVERRIDE = 15;
    public const CHAR_DIRECTION_POP_DIRECTIONAL_FORMAT = 16;
    public const CHAR_DIRECTION_DIR_NON_SPACING_MARK = 17;
    public const CHAR_DIRECTION_BOUNDARY_NEUTRAL = 18;
    public const CHAR_DIRECTION_FIRST_STRONG_ISOLATE = 19;
    public const CHAR_DIRECTION_LEFT_TO_RIGHT_ISOLATE = 20;
    public const CHAR_DIRECTION_RIGHT_TO_LEFT_ISOLATE = 21;
    public const CHAR_DIRECTION_POP_DIRECTIONAL_ISOLATE = 22;
    public const CHAR_DIRECTION_CHAR_DIRECTION_COUNT = 23;
    public const BLOCK_CODE_NO_BLOCK = 0;
    public const BLOCK_CODE_BASIC_LATIN = 1;
    public const BLOCK_CODE_LATIN_1_SUPPLEMENT = 2;
    public const BLOCK_CODE_LATIN_EXTENDED_A = 3;
    public const BLOCK_CODE_LATIN_EXTENDED_B = 4;
    public const BLOCK_CODE_IPA_EXTENSIONS = 5;
    public const BLOCK_CODE_SPACING_MODIFIER_LETTERS = 6;
    public const BLOCK_CODE_COMBINING_DIACRITICAL_MARKS = 7;
    public const BLOCK_CODE_GREEK = 8;
    public const BLOCK_CODE_CYRILLIC = 9;
    public const BLOCK_CODE_ARMENIAN = 10;
    public const BLOCK_CODE_HEBREW = 11;
    public const BLOCK_CODE_ARABIC = 12;
    public const BLOCK_CODE_SYRIAC = 13;
    public const BLOCK_CODE_THAANA = 14;
    public const BLOCK_CODE_DEVANAGARI = 15;
    public const BLOCK_CODE_BENGALI = 16;
    public const BLOCK_CODE_GURMUKHI = 17;
    public const BLOCK_CODE_GUJARATI = 18;
    public const BLOCK_CODE_ORIYA = 19;
    public const BLOCK_CODE_TAMIL = 20;
    public const BLOCK_CODE_TELUGU = 21;
    public const BLOCK_CODE_KANNADA = 22;
    public const BLOCK_CODE_MALAYALAM = 23;
    public const BLOCK_CODE_SINHALA = 24;
    public const BLOCK_CODE_THAI = 25;
    public const BLOCK_CODE_LAO = 26;
    public const BLOCK_CODE_TIBETAN = 27;
    public const BLOCK_CODE_MYANMAR = 28;
    public const BLOCK_CODE_GEORGIAN = 29;
    public const BLOCK_CODE_HANGUL_JAMO = 30;
    public const BLOCK_CODE_ETHIOPIC = 31;
    public const BLOCK_CODE_CHEROKEE = 32;
    public const BLOCK_CODE_UNIFIED_CANADIAN_ABORIGINAL_SYLLABICS = 33;
    public const BLOCK_CODE_OGHAM = 34;
    public const BLOCK_CODE_RUNIC = 35;
    public const BLOCK_CODE_KHMER = 36;
    public const BLOCK_CODE_MONGOLIAN = 37;
    public const BLOCK_CODE_LATIN_EXTENDED_ADDITIONAL = 38;
    public const BLOCK_CODE_GREEK_EXTENDED = 39;
    public const BLOCK_CODE_GENERAL_PUNCTUATION = 40;
    public const BLOCK_CODE_SUPERSCRIPTS_AND_SUBSCRIPTS = 41;
    public const BLOCK_CODE_CURRENCY_SYMBOLS = 42;
    public const BLOCK_CODE_COMBINING_MARKS_FOR_SYMBOLS = 43;
    public const BLOCK_CODE_LETTERLIKE_SYMBOLS = 44;
    public const BLOCK_CODE_NUMBER_FORMS = 45;
    public const BLOCK_CODE_ARROWS = 46;
    public const BLOCK_CODE_MATHEMATICAL_OPERATORS = 47;
    public const BLOCK_CODE_MISCELLANEOUS_TECHNICAL = 48;
    public const BLOCK_CODE_CONTROL_PICTURES = 49;
    public const BLOCK_CODE_OPTICAL_CHARACTER_RECOGNITION = 50;
    public const BLOCK_CODE_ENCLOSED_ALPHANUMERICS = 51;
    public const BLOCK_CODE_BOX_DRAWING = 52;
    public const BLOCK_CODE_BLOCK_ELEMENTS = 53;
    public const BLOCK_CODE_GEOMETRIC_SHAPES = 54;
    public const BLOCK_CODE_MISCELLANEOUS_SYMBOLS = 55;
    public const BLOCK_CODE_DINGBATS = 56;
    public const BLOCK_CODE_BRAILLE_PATTERNS = 57;
    public const BLOCK_CODE_CJK_RADICALS_SUPPLEMENT = 58;
    public const BLOCK_CODE_KANGXI_RADICALS = 59;
    public const BLOCK_CODE_IDEOGRAPHIC_DESCRIPTION_CHARACTERS = 60;
    public const BLOCK_CODE_CJK_SYMBOLS_AND_PUNCTUATION = 61;
    public const BLOCK_CODE_HIRAGANA = 62;
    public const BLOCK_CODE_KATAKANA = 63;
    public const BLOCK_CODE_BOPOMOFO = 64;
    public const BLOCK_CODE_HANGUL_COMPATIBILITY_JAMO = 65;
    public const BLOCK_CODE_KANBUN = 66;
    public const BLOCK_CODE_BOPOMOFO_EXTENDED = 67;
    public const BLOCK_CODE_ENCLOSED_CJK_LETTERS_AND_MONTHS = 68;
    public const BLOCK_CODE_CJK_COMPATIBILITY = 69;
    public const BLOCK_CODE_CJK_UNIFIED_IDEOGRAPHS_EXTENSION_A = 70;
    public const BLOCK_CODE_CJK_UNIFIED_IDEOGRAPHS = 71;
    public const BLOCK_CODE_YI_SYLLABLES = 72;
    public const BLOCK_CODE_YI_RADICALS = 73;
    public const BLOCK_CODE_HANGUL_SYLLABLES = 74;
    public const BLOCK_CODE_HIGH_SURROGATES = 75;
    public const BLOCK_CODE_HIGH_PRIVATE_USE_SURROGATES = 76;
    public const BLOCK_CODE_LOW_SURROGATES = 77;
    public const BLOCK_CODE_PRIVATE_USE_AREA = 78;
    public const BLOCK_CODE_PRIVATE_USE = 78;
    public const BLOCK_CODE_CJK_COMPATIBILITY_IDEOGRAPHS = 79;
    public const BLOCK_CODE_ALPHABETIC_PRESENTATION_FORMS = 80;
    public const BLOCK_CODE_ARABIC_PRESENTATION_FORMS_A = 81;
    public const BLOCK_CODE_COMBINING_HALF_MARKS = 82;
    public const BLOCK_CODE_CJK_COMPATIBILITY_FORMS = 83;
    public const BLOCK_CODE_SMALL_FORM_VARIANTS = 84;
    public const BLOCK_CODE_ARABIC_PRESENTATION_FORMS_B = 85;
    public const BLOCK_CODE_SPECIALS = 86;
    public const BLOCK_CODE_HALFWIDTH_AND_FULLWIDTH_FORMS = 87;
    public const BLOCK_CODE_OLD_ITALIC = 88;
    public const BLOCK_CODE_GOTHIC = 89;
    public const BLOCK_CODE_DESERET = 90;
    public const BLOCK_CODE_BYZANTINE_MUSICAL_SYMBOLS = 91;
    public const BLOCK_CODE_MUSICAL_SYMBOLS = 92;
    public const BLOCK_CODE_MATHEMATICAL_ALPHANUMERIC_SYMBOLS = 93;
    public const BLOCK_CODE_CJK_UNIFIED_IDEOGRAPHS_EXTENSION_B = 94;
    public const BLOCK_CODE_CJK_COMPATIBILITY_IDEOGRAPHS_SUPPLEMENT = 95;
    public const BLOCK_CODE_TAGS = 96;
    public const BLOCK_CODE_CYRILLIC_SUPPLEMENT = 97;
    public const BLOCK_CODE_CYRILLIC_SUPPLEMENTARY = 97;
    public const BLOCK_CODE_TAGALOG = 98;
    public const BLOCK_CODE_HANUNOO = 99;
    public const BLOCK_CODE_BUHID = 100;
    public const BLOCK_CODE_TAGBANWA = 101;
    public const BLOCK_CODE_MISCELLANEOUS_MATHEMATICAL_SYMBOLS_A = 102;
    public const BLOCK_CODE_SUPPLEMENTAL_ARROWS_A = 103;
    public const BLOCK_CODE_SUPPLEMENTAL_ARROWS_B = 104;
    public const BLOCK_CODE_MISCELLANEOUS_MATHEMATICAL_SYMBOLS_B = 105;
    public const BLOCK_CODE_SUPPLEMENTAL_MATHEMATICAL_OPERATORS = 106;
    public const BLOCK_CODE_KATAKANA_PHONETIC_EXTENSIONS = 107;
    public const BLOCK_CODE_VARIATION_SELECTORS = 108;
    public const BLOCK_CODE_SUPPLEMENTARY_PRIVATE_USE_AREA_A = 109;
    public const BLOCK_CODE_SUPPLEMENTARY_PRIVATE_USE_AREA_B = 110;
    public const BLOCK_CODE_LIMBU = 111;
    public const BLOCK_CODE_TAI_LE = 112;
    public const BLOCK_CODE_KHMER_SYMBOLS = 113;
    public const BLOCK_CODE_PHONETIC_EXTENSIONS = 114;
    public const BLOCK_CODE_MISCELLANEOUS_SYMBOLS_AND_ARROWS = 115;
    public const BLOCK_CODE_YIJING_HEXAGRAM_SYMBOLS = 116;
    public const BLOCK_CODE_LINEAR_B_SYLLABARY = 117;
    public const BLOCK_CODE_LINEAR_B_IDEOGRAMS = 118;
    public const BLOCK_CODE_AEGEAN_NUMBERS = 119;
    public const BLOCK_CODE_UGARITIC = 120;
    public const BLOCK_CODE_SHAVIAN = 121;
    public const BLOCK_CODE_OSMANYA = 122;
    public const BLOCK_CODE_CYPRIOT_SYLLABARY = 123;
    public const BLOCK_CODE_TAI_XUAN_JING_SYMBOLS = 124;
    public const BLOCK_CODE_VARIATION_SELECTORS_SUPPLEMENT = 125;
    public const BLOCK_CODE_ANCIENT_GREEK_MUSICAL_NOTATION = 126;
    public const BLOCK_CODE_ANCIENT_GREEK_NUMBERS = 127;
    public const BLOCK_CODE_ARABIC_SUPPLEMENT = 128;
    public const BLOCK_CODE_BUGINESE = 129;
    public const BLOCK_CODE_CJK_STROKES = 130;
    public const BLOCK_CODE_COMBINING_DIACRITICAL_MARKS_SUPPLEMENT = 131;
    public const BLOCK_CODE_COPTIC = 132;
    public const BLOCK_CODE_ETHIOPIC_EXTENDED = 133;
    public const BLOCK_CODE_ETHIOPIC_SUPPLEMENT = 134;
    public const BLOCK_CODE_GEORGIAN_SUPPLEMENT = 135;
    public const BLOCK_CODE_GLAGOLITIC = 136;
    public const BLOCK_CODE_KHAROSHTHI = 137;
    public const BLOCK_CODE_MODIFIER_TONE_LETTERS = 138;
    public const BLOCK_CODE_NEW_TAI_LUE = 139;
    public const BLOCK_CODE_OLD_PERSIAN = 140;
    public const BLOCK_CODE_PHONETIC_EXTENSIONS_SUPPLEMENT = 141;
    public const BLOCK_CODE_SUPPLEMENTAL_PUNCTUATION = 142;
    public const BLOCK_CODE_SYLOTI_NAGRI = 143;
    public const BLOCK_CODE_TIFINAGH = 144;
    public const BLOCK_CODE_VERTICAL_FORMS = 145;
    public const BLOCK_CODE_NKO = 146;
    public const BLOCK_CODE_BALINESE = 147;
    public const BLOCK_CODE_LATIN_EXTENDED_C = 148;
    public const BLOCK_CODE_LATIN_EXTENDED_D = 149;
    public const BLOCK_CODE_PHAGS_PA = 150;
    public const BLOCK_CODE_PHOENICIAN = 151;
    public const BLOCK_CODE_CUNEIFORM = 152;
    public const BLOCK_CODE_CUNEIFORM_NUMBERS_AND_PUNCTUATION = 153;
    public const BLOCK_CODE_COUNTING_ROD_NUMERALS = 154;
    public const BLOCK_CODE_SUNDANESE = 155;
    public const BLOCK_CODE_LEPCHA = 156;
    public const BLOCK_CODE_OL_CHIKI = 157;
    public const BLOCK_CODE_CYRILLIC_EXTENDED_A = 158;
    public const BLOCK_CODE_VAI = 159;
    public const BLOCK_CODE_CYRILLIC_EXTENDED_B = 160;
    public const BLOCK_CODE_SAURASHTRA = 161;
    public const BLOCK_CODE_KAYAH_LI = 162;
    public const BLOCK_CODE_REJANG = 163;
    public const BLOCK_CODE_CHAM = 164;
    public const BLOCK_CODE_ANCIENT_SYMBOLS = 165;
    public const BLOCK_CODE_PHAISTOS_DISC = 166;
    public const BLOCK_CODE_LYCIAN = 167;
    public const BLOCK_CODE_CARIAN = 168;
    public const BLOCK_CODE_LYDIAN = 169;
    public const BLOCK_CODE_MAHJONG_TILES = 170;
    public const BLOCK_CODE_DOMINO_TILES = 171;
    public const BLOCK_CODE_SAMARITAN = 172;
    public const BLOCK_CODE_UNIFIED_CANADIAN_ABORIGINAL_SYLLABICS_EXTENDED = 173;
    public const BLOCK_CODE_TAI_THAM = 174;
    public const BLOCK_CODE_VEDIC_EXTENSIONS = 175;
    public const BLOCK_CODE_LISU = 176;
    public const BLOCK_CODE_BAMUM = 177;
    public const BLOCK_CODE_COMMON_INDIC_NUMBER_FORMS = 178;
    public const BLOCK_CODE_DEVANAGARI_EXTENDED = 179;
    public const BLOCK_CODE_HANGUL_JAMO_EXTENDED_A = 180;
    public const BLOCK_CODE_JAVANESE = 181;
    public const BLOCK_CODE_MYANMAR_EXTENDED_A = 182;
    public const BLOCK_CODE_TAI_VIET = 183;
    public const BLOCK_CODE_MEETEI_MAYEK = 184;
    public const BLOCK_CODE_HANGUL_JAMO_EXTENDED_B = 185;
    public const BLOCK_CODE_IMPERIAL_ARAMAIC = 186;
    public const BLOCK_CODE_OLD_SOUTH_ARABIAN = 187;
    public const BLOCK_CODE_AVESTAN = 188;
    public const BLOCK_CODE_INSCRIPTIONAL_PARTHIAN = 189;
    public const BLOCK_CODE_INSCRIPTIONAL_PAHLAVI = 190;
    public const BLOCK_CODE_OLD_TURKIC = 191;
    public const BLOCK_CODE_RUMI_NUMERAL_SYMBOLS = 192;
    public const BLOCK_CODE_KAITHI = 193;
    public const BLOCK_CODE_EGYPTIAN_HIEROGLYPHS = 194;
    public const BLOCK_CODE_ENCLOSED_ALPHANUMERIC_SUPPLEMENT = 195;
    public const BLOCK_CODE_ENCLOSED_IDEOGRAPHIC_SUPPLEMENT = 196;
    public const BLOCK_CODE_CJK_UNIFIED_IDEOGRAPHS_EXTENSION_C = 197;
    public const BLOCK_CODE_MANDAIC = 198;
    public const BLOCK_CODE_BATAK = 199;
    public const BLOCK_CODE_ETHIOPIC_EXTENDED_A = 200;
    public const BLOCK_CODE_BRAHMI = 201;
    public const BLOCK_CODE_BAMUM_SUPPLEMENT = 202;
    public const BLOCK_CODE_KANA_SUPPLEMENT = 203;
    public const BLOCK_CODE_PLAYING_CARDS = 204;
    public const BLOCK_CODE_MISCELLANEOUS_SYMBOLS_AND_PICTOGRAPHS = 205;
    public const BLOCK_CODE_EMOTICONS = 206;
    public const BLOCK_CODE_TRANSPORT_AND_MAP_SYMBOLS = 207;
    public const BLOCK_CODE_ALCHEMICAL_SYMBOLS = 208;
    public const BLOCK_CODE_CJK_UNIFIED_IDEOGRAPHS_EXTENSION_D = 209;
    public const BLOCK_CODE_ARABIC_EXTENDED_A = 210;
    public const BLOCK_CODE_ARABIC_MATHEMATICAL_ALPHABETIC_SYMBOLS = 211;
    public const BLOCK_CODE_CHAKMA = 212;
    public const BLOCK_CODE_MEETEI_MAYEK_EXTENSIONS = 213;
    public const BLOCK_CODE_MEROITIC_CURSIVE = 214;
    public const BLOCK_CODE_MEROITIC_HIEROGLYPHS = 215;
    public const BLOCK_CODE_MIAO = 216;
    public const BLOCK_CODE_SHARADA = 217;
    public const BLOCK_CODE_SORA_SOMPENG = 218;
    public const BLOCK_CODE_SUNDANESE_SUPPLEMENT = 219;
    public const BLOCK_CODE_TAKRI = 220;
    public const BLOCK_CODE_BASSA_VAH = 221;
    public const BLOCK_CODE_CAUCASIAN_ALBANIAN = 222;
    public const BLOCK_CODE_COPTIC_EPACT_NUMBERS = 223;
    public const BLOCK_CODE_COMBINING_DIACRITICAL_MARKS_EXTENDED = 224;
    public const BLOCK_CODE_DUPLOYAN = 225;
    public const BLOCK_CODE_ELBASAN = 226;
    public const BLOCK_CODE_GEOMETRIC_SHAPES_EXTENDED = 227;
    public const BLOCK_CODE_GRANTHA = 228;
    public const BLOCK_CODE_KHOJKI = 229;
    public const BLOCK_CODE_KHUDAWADI = 230;
    public const BLOCK_CODE_LATIN_EXTENDED_E = 231;
    public const BLOCK_CODE_LINEAR_A = 232;
    public const BLOCK_CODE_MAHAJANI = 233;
    public const BLOCK_CODE_MANICHAEAN = 234;
    public const BLOCK_CODE_MENDE_KIKAKUI = 235;
    public const BLOCK_CODE_MODI = 236;
    public const BLOCK_CODE_MRO = 237;
    public const BLOCK_CODE_MYANMAR_EXTENDED_B = 238;
    public const BLOCK_CODE_NABATAEAN = 239;
    public const BLOCK_CODE_OLD_NORTH_ARABIAN = 240;
    public const BLOCK_CODE_OLD_PERMIC = 241;
    public const BLOCK_CODE_ORNAMENTAL_DINGBATS = 242;
    public const BLOCK_CODE_PAHAWH_HMONG = 243;
    public const BLOCK_CODE_PALMYRENE = 244;
    public const BLOCK_CODE_PAU_CIN_HAU = 245;
    public const BLOCK_CODE_PSALTER_PAHLAVI = 246;
    public const BLOCK_CODE_SHORTHAND_FORMAT_CONTROLS = 247;
    public const BLOCK_CODE_SIDDHAM = 248;
    public const BLOCK_CODE_SINHALA_ARCHAIC_NUMBERS = 249;
    public const BLOCK_CODE_SUPPLEMENTAL_ARROWS_C = 250;
    public const BLOCK_CODE_TIRHUTA = 251;
    public const BLOCK_CODE_WARANG_CITI = 252;
    public const BLOCK_CODE_COUNT = 309;
    public const BLOCK_CODE_INVALID_CODE = -1;
    public const BPT_NONE = 0;
    public const BPT_OPEN = 1;
    public const BPT_CLOSE = 2;
    public const BPT_COUNT = 3;
    public const EA_NEUTRAL = 0;
    public const EA_AMBIGUOUS = 1;
    public const EA_HALFWIDTH = 2;
    public const EA_FULLWIDTH = 3;
    public const EA_NARROW = 4;
    public const EA_WIDE = 5;
    public const EA_COUNT = 6;
    public const UNICODE_CHAR_NAME = 0;
    public const UNICODE_10_CHAR_NAME = 1;
    public const EXTENDED_CHAR_NAME = 2;
    public const CHAR_NAME_ALIAS = 3;
    public const CHAR_NAME_CHOICE_COUNT = 4;
    public const SHORT_PROPERTY_NAME = 0;
    public const LONG_PROPERTY_NAME = 1;
    public const PROPERTY_NAME_CHOICE_COUNT = 2;
    public const DT_NONE = 0;
    public const DT_CANONICAL = 1;
    public const DT_COMPAT = 2;
    public const DT_CIRCLE = 3;
    public const DT_FINAL = 4;
    public const DT_FONT = 5;
    public const DT_FRACTION = 6;
    public const DT_INITIAL = 7;
    public const DT_ISOLATED = 8;
    public const DT_MEDIAL = 9;
    public const DT_NARROW = 10;
    public const DT_NOBREAK = 11;
    public const DT_SMALL = 12;
    public const DT_SQUARE = 13;
    public const DT_SUB = 14;
    public const DT_SUPER = 15;
    public const DT_VERTICAL = 16;
    public const DT_WIDE = 17;
    public const DT_COUNT = 18;
    public const JT_NON_JOINING = 0;
    public const JT_JOIN_CAUSING = 1;
    public const JT_DUAL_JOINING = 2;
    public const JT_LEFT_JOINING = 3;
    public const JT_RIGHT_JOINING = 4;
    public const JT_TRANSPARENT = 5;
    public const JT_COUNT = 6;
    public const JG_NO_JOINING_GROUP = 0;
    public const JG_AIN = 1;
    public const JG_ALAPH = 2;
    public const JG_ALEF = 3;
    public const JG_BEH = 4;
    public const JG_BETH = 5;
    public const JG_DAL = 6;
    public const JG_DALATH_RISH = 7;
    public const JG_E = 8;
    public const JG_FEH = 9;
    public const JG_FINAL_SEMKATH = 10;
    public const JG_GAF = 11;
    public const JG_GAMAL = 12;
    public const JG_HAH = 13;
    public const JG_TEH_MARBUTA_GOAL = 14;
    public const JG_HAMZA_ON_HEH_GOAL = 14;
    public const JG_HE = 15;
    public const JG_HEH = 16;
    public const JG_HEH_GOAL = 17;
    public const JG_HETH = 18;
    public const JG_KAF = 19;
    public const JG_KAPH = 20;
    public const JG_KNOTTED_HEH = 21;
    public const JG_LAM = 22;
    public const JG_LAMADH = 23;
    public const JG_MEEM = 24;
    public const JG_MIM = 25;
    public const JG_NOON = 26;
    public const JG_NUN = 27;
    public const JG_PE = 28;
    public const JG_QAF = 29;
    public const JG_QAPH = 30;
    public const JG_REH = 31;
    public const JG_REVERSED_PE = 32;
    public const JG_SAD = 33;
    public const JG_SADHE = 34;
    public const JG_SEEN = 35;
    public const JG_SEMKATH = 36;
    public const JG_SHIN = 37;
    public const JG_SWASH_KAF = 38;
    public const JG_SYRIAC_WAW = 39;
    public const JG_TAH = 40;
    public const JG_TAW = 41;
    public const JG_TEH_MARBUTA = 42;
    public const JG_TETH = 43;
    public const JG_WAW = 44;
    public const JG_YEH = 45;
    public const JG_YEH_BARREE = 46;
    public const JG_YEH_WITH_TAIL = 47;
    public const JG_YUDH = 48;
    public const JG_YUDH_HE = 49;
    public const JG_ZAIN = 50;
    public const JG_FE = 51;
    public const JG_KHAPH = 52;
    public const JG_ZHAIN = 53;
    public const JG_BURUSHASKI_YEH_BARREE = 54;
    public const JG_FARSI_YEH = 55;
    public const JG_NYA = 56;
    public const JG_ROHINGYA_YEH = 57;
    public const JG_MANICHAEAN_ALEPH = 58;
    public const JG_MANICHAEAN_AYIN = 59;
    public const JG_MANICHAEAN_BETH = 60;
    public const JG_MANICHAEAN_DALETH = 61;
    public const JG_MANICHAEAN_DHAMEDH = 62;
    public const JG_MANICHAEAN_FIVE = 63;
    public const JG_MANICHAEAN_GIMEL = 64;
    public const JG_MANICHAEAN_HETH = 65;
    public const JG_MANICHAEAN_HUNDRED = 66;
    public const JG_MANICHAEAN_KAPH = 67;
    public const JG_MANICHAEAN_LAMEDH = 68;
    public const JG_MANICHAEAN_MEM = 69;
    public const JG_MANICHAEAN_NUN = 70;
    public const JG_MANICHAEAN_ONE = 71;
    public const JG_MANICHAEAN_PE = 72;
    public const JG_MANICHAEAN_QOPH = 73;
    public const JG_MANICHAEAN_RESH = 74;
    public const JG_MANICHAEAN_SADHE = 75;
    public const JG_MANICHAEAN_SAMEKH = 76;
    public const JG_MANICHAEAN_TAW = 77;
    public const JG_MANICHAEAN_TEN = 78;
    public const JG_MANICHAEAN_TETH = 79;
    public const JG_MANICHAEAN_THAMEDH = 80;
    public const JG_MANICHAEAN_TWENTY = 81;
    public const JG_MANICHAEAN_WAW = 82;
    public const JG_MANICHAEAN_YODH = 83;
    public const JG_MANICHAEAN_ZAYIN = 84;
    public const JG_STRAIGHT_WAW = 85;
    public const JG_COUNT = 102;
    public const GCB_OTHER = 0;
    public const GCB_CONTROL = 1;
    public const GCB_CR = 2;
    public const GCB_EXTEND = 3;
    public const GCB_L = 4;
    public const GCB_LF = 5;
    public const GCB_LV = 6;
    public const GCB_LVT = 7;
    public const GCB_T = 8;
    public const GCB_V = 9;
    public const GCB_SPACING_MARK = 10;
    public const GCB_PREPEND = 11;
    public const GCB_REGIONAL_INDICATOR = 12;
    public const GCB_COUNT = 18;
    public const WB_OTHER = 0;
    public const WB_ALETTER = 1;
    public const WB_FORMAT = 2;
    public const WB_KATAKANA = 3;
    public const WB_MIDLETTER = 4;
    public const WB_MIDNUM = 5;
    public const WB_NUMERIC = 6;
    public const WB_EXTENDNUMLET = 7;
    public const WB_CR = 8;
    public const WB_EXTEND = 9;
    public const WB_LF = 10;
    public const WB_MIDNUMLET = 11;
    public const WB_NEWLINE = 12;
    public const WB_REGIONAL_INDICATOR = 13;
    public const WB_HEBREW_LETTER = 14;
    public const WB_SINGLE_QUOTE = 15;
    public const WB_DOUBLE_QUOTE = 16;
    public const WB_COUNT = 23;
    public const SB_OTHER = 0;
    public const SB_ATERM = 1;
    public const SB_CLOSE = 2;
    public const SB_FORMAT = 3;
    public const SB_LOWER = 4;
    public const SB_NUMERIC = 5;
    public const SB_OLETTER = 6;
    public const SB_SEP = 7;
    public const SB_SP = 8;
    public const SB_STERM = 9;
    public const SB_UPPER = 10;
    public const SB_CR = 11;
    public const SB_EXTEND = 12;
    public const SB_LF = 13;
    public const SB_SCONTINUE = 14;
    public const SB_COUNT = 15;
    public const LB_UNKNOWN = 0;
    public const LB_AMBIGUOUS = 1;
    public const LB_ALPHABETIC = 2;
    public const LB_BREAK_BOTH = 3;
    public const LB_BREAK_AFTER = 4;
    public const LB_BREAK_BEFORE = 5;
    public const LB_MANDATORY_BREAK = 6;
    public const LB_CONTINGENT_BREAK = 7;
    public const LB_CLOSE_PUNCTUATION = 8;
    public const LB_COMBINING_MARK = 9;
    public const LB_CARRIAGE_RETURN = 10;
    public const LB_EXCLAMATION = 11;
    public const LB_GLUE = 12;
    public const LB_HYPHEN = 13;
    public const LB_IDEOGRAPHIC = 14;
    public const LB_INSEPARABLE = 15;
    public const LB_INSEPERABLE = 15;
    public const LB_INFIX_NUMERIC = 16;
    public const LB_LINE_FEED = 17;
    public const LB_NONSTARTER = 18;
    public const LB_NUMERIC = 19;
    public const LB_OPEN_PUNCTUATION = 20;
    public const LB_POSTFIX_NUMERIC = 21;
    public const LB_PREFIX_NUMERIC = 22;
    public const LB_QUOTATION = 23;
    public const LB_COMPLEX_CONTEXT = 24;
    public const LB_SURROGATE = 25;
    public const LB_SPACE = 26;
    public const LB_BREAK_SYMBOLS = 27;
    public const LB_ZWSPACE = 28;
    public const LB_NEXT_LINE = 29;
    public const LB_WORD_JOINER = 30;
    public const LB_H2 = 31;
    public const LB_H3 = 32;
    public const LB_JL = 33;
    public const LB_JT = 34;
    public const LB_JV = 35;
    public const LB_CLOSE_PARENTHESIS = 36;
    public const LB_CONDITIONAL_JAPANESE_STARTER = 37;
    public const LB_HEBREW_LETTER = 38;
    public const LB_REGIONAL_INDICATOR = 39;
    public const LB_COUNT = 43;
    public const NT_NONE = 0;
    public const NT_DECIMAL = 1;
    public const NT_DIGIT = 2;
    public const NT_NUMERIC = 3;
    public const NT_COUNT = 4;
    public const HST_NOT_APPLICABLE = 0;
    public const HST_LEADING_JAMO = 1;
    public const HST_VOWEL_JAMO = 2;
    public const HST_TRAILING_JAMO = 3;
    public const HST_LV_SYLLABLE = 4;
    public const HST_LVT_SYLLABLE = 5;
    public const HST_COUNT = 6;
    public const NO_NUMERIC_VALUE = -123456789;
    public const PROPERTY_IDS_UNARY_OPERATOR = 72;
    public const PROPERTY_ID_COMPAT_MATH_START = 73;
    public const PROPERTY_ID_COMPAT_MATH_CONTINUE = 74;

    /**
     * @pure
     */
    public static function hasBinaryProperty(int|string $codepoint, int $property): null|bool
    {
    }

    public static function charAge(int|string $codepoint): null|array
    {
    }

    public static function charDigitValue(int|string $codepoint): null|int
    {
    }

    public static function charDirection(int|string $codepoint): null|int
    {
    }

    public static function charFromName(string $name, int $type = IntlChar::UNICODE_CHAR_NAME): null|int
    {
    }

    public static function charMirror(int|string $codepoint): string|int|null
    {
    }

    public static function charName(int|string $codepoint, int $type = IntlChar::UNICODE_CHAR_NAME): null|string
    {
    }

    public static function charType(int|string $codepoint): null|int
    {
    }

    public static function chr(int|string $codepoint): null|string
    {
    }

    public static function digit(int|string $codepoint, int $base = 10): int|false|null
    {
    }

    public static function enumCharNames(
        int|string $start,
        int|string $end,
        callable $callback,
        int $type = IntlChar::UNICODE_CHAR_NAME,
    ): null|bool {
    }

    public static function enumCharTypes(callable $callback): void
    {
    }

    public static function foldCase(int|string $codepoint, int $options = IntlChar::FOLD_CASE_DEFAULT): string|int|null
    {
    }

    public static function forDigit(int $digit, int $base = 10): int
    {
    }

    public static function getBidiPairedBracket(int|string $codepoint): string|int|null
    {
    }

    public static function getBlockCode(int|string $codepoint): null|int
    {
    }

    public static function getCombiningClass(int|string $codepoint): null|int
    {
    }

    public static function getFC_NFKC_Closure(int|string $codepoint): string|false|null
    {
    }

    public static function getIntPropertyMaxValue(int $property): int
    {
    }

    public static function getIntPropertyMinValue(int $property): int
    {
    }

    public static function getIntPropertyValue(int|string $codepoint, int $property): null|int
    {
    }

    public static function getNumericValue(int|string $codepoint): null|float
    {
    }

    public static function getPropertyEnum(string $alias): int
    {
    }

    public static function getPropertyName(int $property, int $type = IntlChar::LONG_PROPERTY_NAME): string|false
    {
    }

    public static function getPropertyValueEnum(int $property, string $name): int
    {
    }

    public static function getPropertyValueName(
        int $property,
        int $value,
        int $type = IntlChar::LONG_PROPERTY_NAME,
    ): string|false {
    }

    /**
     * @return array{0: int<0, max>, 1: int<0, max>, 2: int<0, max>, 3: int<0, max>}
     */
    public static function getUnicodeVersion(): array
    {
    }

    public static function isalnum(int|string $codepoint): null|bool
    {
    }

    public static function isalpha(int|string $codepoint): null|bool
    {
    }

    public static function isbase(int|string $codepoint): null|bool
    {
    }

    public static function isblank(int|string $codepoint): null|bool
    {
    }

    public static function iscntrl(int|string $codepoint): null|bool
    {
    }

    public static function isdefined(int|string $codepoint): null|bool
    {
    }

    public static function isdigit(int|string $codepoint): null|bool
    {
    }

    public static function isgraph(int|string $codepoint): null|bool
    {
    }

    public static function isIDIgnorable(int|string $codepoint): null|bool
    {
    }

    public static function isIDPart(int|string $codepoint): null|bool
    {
    }

    public static function isIDStart(int|string $codepoint): null|bool
    {
    }

    public static function isISOControl(int|string $codepoint): null|bool
    {
    }

    public static function isJavaIDPart(int|string $codepoint): null|bool
    {
    }

    public static function isJavaIDStart(int|string $codepoint): null|bool
    {
    }

    public static function isJavaSpaceChar(int|string $codepoint): null|bool
    {
    }

    public static function islower(int|string $codepoint): null|bool
    {
    }

    public static function isMirrored(int|string $codepoint): null|bool
    {
    }

    public static function isprint(int|string $codepoint): null|bool
    {
    }

    public static function ispunct(int|string $codepoint): null|bool
    {
    }

    public static function isspace(int|string $codepoint): null|bool
    {
    }

    public static function istitle(int|string $codepoint): null|bool
    {
    }

    public static function isUAlphabetic(int|string $codepoint): null|bool
    {
    }

    public static function isULowercase(int|string $codepoint): null|bool
    {
    }

    public static function isupper(int|string $codepoint): null|bool
    {
    }

    public static function isUUppercase(int|string $codepoint): null|bool
    {
    }

    public static function isUWhiteSpace(int|string $codepoint): null|bool
    {
    }

    public static function isWhitespace(int|string $codepoint): null|bool
    {
    }

    public static function isxdigit(int|string $codepoint): null|bool
    {
    }

    public static function ord(int|string $character): null|int
    {
    }

    public static function tolower(int|string $codepoint): string|int|null
    {
    }

    public static function totitle(int|string $codepoint): string|int|null
    {
    }

    public static function toupper(int|string $codepoint): string|int|null
    {
    }
}

class IntlDatePatternGenerator
{
    public function __construct(null|string $locale = null) {}

    public static function create(null|string $locale = null): null|IntlDatePatternGenerator
    {
    }

    public function getBestPattern(string $skeleton): string|false
    {
    }
}
