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

    /**
     * @since 8.4
     */
    public function parseToCalendar(string $string, &$offset = null): int|float|false
    {
    }
}

class ResourceBundle implements IteratorAggregate, Countable
{
    /**
     * @link https://www.php.net/manual/en/resourcebundle.create.php
     * @param string $locale <p>Locale for which the resources should be loaded (locale name, e.g. en_CA).</p>
     * @param string $bundle <p>The directory where the data is stored or the name of the .dat file.</p>
     * @param bool $fallback [optional] <p>Whether locale should match exactly or fallback to parent locale is allowed.</p>
     * @pure
     */
    public function __construct(string|null $locale, string|null $bundle, bool $fallback = true) {}

    /**
     * (PHP &gt;= 5.3.2, PECL intl &gt;= 2.0.0)<br/>
     * Create a resource bundle
     * @link https://php.net/manual/en/resourcebundle.create.php
     * @param string $locale <p>
     * Locale for which the resources should be loaded (locale name, e.g. en_CA).
     * </p>
     * @param string $bundle <p>
     * The directory where the data is stored or the name of the .dat file.
     * </p>
     * @param bool $fallback [optional] <p>
     * Whether locale should match exactly or fallback to parent locale is allowed.
     * </p>
     * @return ResourceBundle|null <b>ResourceBundle</b> object or <b>null</b> on error.
     */
    public static function create(string|null $locale, string|null $bundle, bool $fallback = true): null|ResourceBundle
    {
    }

    /**
     * (PHP &gt;= 5.3.2, PECL intl &gt;= 2.0.0)<br/>
     * Get data from the bundle
     * @link https://php.net/manual/en/resourcebundle.get.php
     * @param string|int $index <p>
     * Data index, must be string or integer.
     * </p>
     * @param bool $fallback
     * @return ResourceBundle|array|string|int|null
     * @pure
     */
    public function get(string|int $index, bool $fallback = true)
    {
    }

    /**
     * (PHP &gt;= 5.3.2, PECL intl &gt;= 2.0.0)<br/>
     * Get number of elements in the bundle
     * @link https://php.net/manual/en/resourcebundle.count.php
     * @return int<0,max> number of elements in the bundle.
     * @pure
     */
    public function count(): int
    {
    }

    /**
     * (PHP &gt;= 5.3.2, PECL intl &gt;= 2.0.0)<br/>
     * Get supported locales
     * @link https://php.net/manual/en/resourcebundle.locales.php
     * @param string $bundle <p>
     * Path of ResourceBundle for which to get available locales, or
     * empty string for default locales list.
     * </p>
     * @return array|false the list of locales supported by the bundle.
     */
    public static function getLocales(string $bundle): array|false
    {
    }

    /**
     * (PHP &gt;= 5.3.2, PECL intl &gt;= 2.0.0)<br/>
     * Get bundle's last error code.
     * @link https://php.net/manual/en/resourcebundle.geterrorcode.php
     * @return int error code from last bundle object call.
     *
     * @pure
     */
    public function getErrorCode(): int
    {
    }

    /**
     * (PHP &gt;= 5.3.2, PECL intl &gt;= 2.0.0)<br/>
     * Get bundle's last error message.
     * @link https://php.net/manual/en/resourcebundle.geterrormessage.php
     * @return string error message from last bundle object's call.
     *
     * @pure
     */
    public function getErrorMessage(): string
    {
    }

    /**
     * @return Iterator
     * @since 8.0
     *
     * @pure
     */
    public function getIterator(): Iterator
    {
    }
}

/**
 * @since 5.4
 */
class Transliterator
{
    public const FORWARD = 0;
    public const REVERSE = 1;

    /**
     * Starting 8.2 $id is readonly to unlock subclassing it
     */
    public readonly string $id;

    /**
     * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
     * Private constructor to deny instantiation
     * @link https://php.net/manual/en/transliterator.construct.php
     */
    final private function __construct() {}

    /**
     * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
     * Create a transliterator
     * @link https://php.net/manual/en/transliterator.create.php
     * @param string $id <p>
     * The id.
     * </p>
     * @param int $direction [optional] <p>
     * The direction, defaults to
     * Transliterator::FORWARD.
     * May also be set to
     * Transliterator::REVERSE.
     * </p>
     * @return Transliterator|null a <b>Transliterator</b> object on success,
     * or <b>NULL</b> on failure.
     */
    public static function create(string $id, int $direction = 0): null|Transliterator
    {
    }

    /**
     * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
     * Create transliterator from rules
     * @link https://php.net/manual/en/transliterator.createfromrules.php
     * @param string $rules <p>
     * The rules.
     * </p>
     * @param int $direction [optional] <p>
     * The direction, defaults to
     * {@see Transliterator::FORWARD}.
     * May also be set to
     * {@see Transliterator::REVERSE}.
     * </p>
     * @return Transliterator|null a <b>Transliterator</b> object on success,
     * or <b>NULL</b> on failure.
     */
    public static function createFromRules(string $rules, int $direction = 0): null|Transliterator
    {
    }

    /**
     * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
     * Create an inverse transliterator
     * @link https://php.net/manual/en/transliterator.createinverse.php
     * @return Transliterator|null a <b>Transliterator</b> object on success,
     * or <b>NULL</b> on failure
     *
     * @pure
     */
    public function createInverse(): null|Transliterator
    {
    }

    /**
     * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
     * Get transliterator IDs
     * @link https://php.net/manual/en/transliterator.listids.php
     * @return array|false An array of registered transliterator IDs on success,
     * or <b>FALSE</b> on failure.
     */
    public static function listIDs(): array|false
    {
    }

    /**
     * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
     * Transliterate a string
     * @link https://php.net/manual/en/transliterator.transliterate.php
     * @param string $string <p>
     * The string to be transformed.
     * </p>
     * @param int $start [optional] <p>
     * The start index (in UTF-16 code units) from which the string will start
     * to be transformed, inclusive. Indexing starts at 0. The text before will
     * be left as is.
     * </p>
     * @param int $end [optional] <p>
     * The end index (in UTF-16 code units) until which the string will be
     * transformed, exclusive. Indexing starts at 0. The text after will be
     * left as is.
     * </p>
     * @return string|false The transfomed string on success, or <b>FALSE</b> on failure.
     *
     * @pure
     */
    public function transliterate(string $string, int $start = 0, int $end = -1): string|false
    {
    }

    /**
     * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
     * Get last error code
     * @link https://php.net/manual/en/transliterator.geterrorcode.php
     * @return int|false The error code on success,
     * or <b>FALSE</b> if none exists, or on failure.
     *
     * @pure
     */
    public function getErrorCode(): int|false
    {
    }

    /**
     * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
     * Get last error message
     * @link https://php.net/manual/en/transliterator.geterrormessage.php
     * @return string|false The error code on success,
     * or <b>FALSE</b> if none exists, or on failure.
     *
     * @pure
     */
    public function getErrorMessage(): string|false
    {
    }
}

/**
 * @link https://php.net/manual/en/class.spoofchecker.php
 */
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

    /**
     * @since 8.4
     */
    public const IGNORE_SPACE = 1;

    /**
     * @since 8.4
     */
    public const CASE_INSENSITIVE = 2;

    /**
     * @since 8.4
     */
    public const ADD_CASE_MAPPINGS = 4;

    /**
     * @since 8.4
     */
    public const SIMPLE_CASE_INSENSITIVE = 6;

    /**
     * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
     * Constructor
     * @link https://php.net/manual/en/spoofchecker.construct.php
     *
     * @pure
     */
    public function __construct() {}

    /**
     * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
     * Checks if a given text contains any suspicious characters
     * @link https://php.net/manual/en/spoofchecker.issuspicious.php
     * @param string $string <p>
     * </p>
     * @param string &$errorCode [optional] <p>
     * </p>
     * @return bool
     */
    public function isSuspicious(string $string, &$errorCode = null): bool
    {
    }

    /**
     * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
     * Checks if a given text contains any confusable characters
     * @link https://php.net/manual/en/spoofchecker.areconfusable.php
     * @param string $string1 <p>
     * </p>
     * @param string $string2 <p>
     * </p>
     * @param int &$errorCode [optional] <p>
     * </p>
     * @return bool
     */
    public function areConfusable(string $string1, string $string2, &$errorCode = null): bool
    {
    }

    /**
     * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
     * Locales to use when running checks
     * @link https://php.net/manual/en/spoofchecker.setallowedlocales.php
     * @param string $locales <p>
     * </p>
     * @return void
     */
    public function setAllowedLocales(string $locales): void
    {
    }

    /**
     * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
     * Set the checks to run
     * @link https://php.net/manual/en/spoofchecker.setchecks.php
     * @param int $checks <p>
     * </p>
     * @return void
     */
    public function setChecks(int $checks): void
    {
    }

    /**
     * @param int $level
     */
    public function setRestrictionLevel(int $level): void
    {
    }

    /**
     * @since 8.4
     */
    public function setAllowedChars(string $pattern, int $patternOptions = 0): void
    {
    }
}

class IntlGregorianCalendar extends IntlCalendar
{
    /**
     * @link https://www.php.net/manual/en/intlgregoriancalendar.construct
     * @param int $timezoneOrYear [optional]
     * @param int $localeOrMonth [optional]
     * @param int $day [optional]
     * @param int $hour [optional]
     * @param int $minute [optional]
     * @param int $second [optional]
     */
    public function __construct($timezoneOrYear, $localeOrMonth, $day, $hour, $minute, $second) {}

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
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

    /**
     * @since 8.3
     */
    public static function createFromDate(int $year, int $month, int $dayOfMonth): static
    {
    }

    /**
     * @since 8.3
     */
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

    /* Methods */

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Add a (signed) amount of time to a field
     * @link https://secure.php.net/manual/en/intlcalendar.add.php
     * @param int $field <p>
     * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}.
     * These are integer values between <em>0</em> and
     * <b>IntlCalendar::FIELD_COUNT</b>.
     * </p>
     * @param int $value <p>The signed amount to add to the current field. If the amount is positive, the instant will be moved forward; if it is negative, the instant wil be moved into the past. The unit is implicit to the field type.
     * For instance, hours for <b>IntlCalendar::FIELD_HOUR_OF_DAY</b>.</p>
     * @return bool Returns TRUE on success or FALSE on failure.
     */
    public function add(int $field, int $value): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Whether this object's time is after that of the passed object
     * https://secure.php.net/manual/en/intlcalendar.after.php
     * @param IntlCalendar $other <p>The calendar whose time will be checked against this object's time.</p>
     * @return bool
     * Returns <b>TRUE</b> if this object's current time is after that of the
     * <em>calendar</em> argument's time. Returns <b>FALSE</b> otherwise.
     * Also returns <b>FALSE</b> on failure. You can use {@link https://secure.php.net/manual/en/intl.configuration.php#ini.intl.use-exceptions exceptions} or
     * {@link https://secure.php.net/manual/en/function.intl-get-error-code.php intl_get_error_code()} to detect error conditions.
     *
     * @pure
     */
    public function after(IntlCalendar $other): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Whether this object's time is before that of the passed object
     * @link https://secure.php.net/manual/en/intlcalendar.before.php
     * @param IntlCalendar $other <p> The calendar whose time will be checked against this object's time.</p>
     * @return bool
     * Returns <b>TRUE</B> if this object's current time is before that of the
     * <em>calendar</em> argument's time. Returns <b>FALSE</b> otherwise.
     * Also returns <b>FALSE</b> on failure. You can use {@link https://secure.php.net/manual/en/intl.configuration.php#ini.intl.use-exceptions exceptions} or
     * {@link https://secure.php.net/manual/en/function.intl-get-error-code.php intl_get_error_code()} to detect error conditions.
     *
     * @pure
     */
    public function before(IntlCalendar $other): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Clear a field or all fields
     * @link https://secure.php.net/manual/en/intlcalendar.clear.php
     * @param int $field [optional] <p>
     * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
     * values between <em>0</em> and
     * <b>IntlCalendar::FIELD_COUNT</b>.
     * </p>
     * @return bool Returns <b>TRUE</b> on success or <b>FALSE</b> on failure. Failure can only occur is invalid arguments are provided.
     */
    public function clear(null|int $field = null): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Private constructor for disallowing instantiation
     * @link https://secure.php.net/manual/en/intlcalendar.construct.php
     */
    private function __construct() {}

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Create a new IntlCalendar
     * @link https://secure.php.net/manual/en/intlcalendar.createinstance.php
     * @param mixed $timezone [optional] <p> <p>
     * The timezone to use.
     * </p>
     *
     * <ul>
     * <li>
     * <p>
     * <b>NULL</b>, in which case the default timezone will be used, as specified in
     * the ini setting {@link https://secure.php.net/manual/en/datetime.configuration.php#ini.date.timezone date.timezone} or
     * through the function  {@link https://secure.php.net/manual/en/function.date-default-timezone-set.php date_default_timezone_set()} and as
     * returned by {@link https://secure.php.net/manual/en/function.date-default-timezone-get.php date_default_timezone_get()}.
     * </p>
     * </li>
     * <li>
     * <p>
     * An {@link https://secure.php.net/manual/en/class.intltimezone.php IntlTimeZone}, which will be used directly.
     * </p>
     * </li>
     * <li>
     * <p>
     * A {@link https://secure.php.net/manual/en/class.datetimezone.php DateTimeZone}. Its identifier will be extracted
     * and an ICU timezone object will be created; the timezone will be backed
     * by ICU's database, not PHP's.
     * </p>
     * </li>
     * <li>
     * <p>
     * A {@link https://secure.php.net/manual/en/language.types.string.php string}, which should be a valid ICU timezone identifier.
     * See  <b>IntlTimeZone::createTimeZoneIDEnumeration()</b>. Raw
     * offsets such as <em>"GMT+08:30"</em> are also accepted.
     * </p>
     * </li>
     * </ul>
     * </p>
     * @param string|null $locale [optional] <p>
     * A locale to use or <b>NULL</b> to use {@link https://secure.php.net/manual/en/intl.configuration.php#ini.intl.default-locale the default locale}.
     * </p>
     * @return IntlCalendar|null
     * The created {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} instance or <b>NULL</b> on
     * failure.
     */
    public static function createInstance($timezone = null, string|null $locale = null): null|IntlCalendar
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Compare time of two IntlCalendar objects for equality
     * @link https://secure.php.net/manual/en/intlcalendar.equals.php
     * @param IntlCalendar $other
     * @return bool <p>
     * Returns <b>TRUE</b> if the current time of both this and the passed in
     * {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} object are the same, or <b>FALSE</b>
     * otherwise. The value <b>FALSE</b> can also be returned on failure. This can only
     * happen if bad arguments are passed in. In any case, the two cases can be
     * distinguished by calling  {@link https://secure.php.net/manual/en/function.intl-get-error-code.php intl_get_error_code()}.
     * </p>
     *
     * @pure
     */
    public function equals(IntlCalendar $other): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Calculate difference between given time and this object's time
     * @link https://secure.php.net/manual/en/intlcalendar.fielddifference.php
     * @param float $timestamp <p>
     * The time against which to compare the quantity represented by the
     * <em>field</em>. For the result to be positive, the time
     * given for this parameter must be ahead of the time of the object the
     * method is being invoked on.
     * </p>
     * @param int $field <p>
     * The field that represents the quantity being compared.
     * </p>
     *
     * <p>
     * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
     * values between <em>0</em> and
     * <b>IntlCalendar::FIELD_COUNT</b>.
     * </p>
     * @return int|false Returns a (signed) difference of time in the unit associated with the
     * specified field or <b>FALSE</b> on failure.
     *
     * @pure
     */
    public function fieldDifference(float $timestamp, int $field): int|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a2)<br/>
     * Create an IntlCalendar from a DateTime object or string
     * @link https://secure.php.net/manual/en/intlcalendar.fromdatetime.php
     * @param mixed $datetime <p>
     * A {@link https://secure.php.net/manual/en/class.datetime.php DateTime} object or a {@link https://secure.php.net/manual/en/language.types.string.php string} that
     * can be passed to  {@link https://secure.php.net/manual/en/datetime.construct.php DateTime::__construct()}.
     * </p>
     * @param $locale [optional]
     * @return IntlCalendar|null
     * The created {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} object or <b>NULL</b> in case of
     * failure. If a {@link https://secure.php.net/manual/en/language.types.string.php string} is passed, any exception that occurs
     * inside the {@link https://secure.php.net/manual/en/class.datetime.php DateTime} constructor is propagated.
     */
    public static function fromDateTime(DateTime|string $datetime, string|null $locale): null|IntlCalendar
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get the value for a field
     * @link https://secure.php.net/manual/en/intlcalendar.get.php
     * @param int $field <p>
     * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
     * values between <em>0</em> and
     * <b>IntlCalendar::FIELD_COUNT</b>.
     * </p>
     * @return int|false An integer with the value of the time field.
     *
     * @pure
     */
    public function get(int $field): int|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * The maximum value for a field, considering the object's current time
     * @link https://secure.php.net/manual/en/intlcalendar.getactualmaximum.php
     * @param int $field <p>
     * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
     * values between <em>0</em> and
     * <b>IntlCalendar::FIELD_COUNT</b>.
     * </p>
     * @return int|false
     * An {@link https://secure.php.net/manual/en/language.types.integer.php int} representing the maximum value in the units associated
     * with the given <em>field</em> or <b>FALSE</b> on failure.
     *
     * @pure
     */
    public function getActualMaximum(int $field): int|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * The minimum value for a field, considering the object's current time
     * @link https://secure.php.net/manual/en/intlcalendar.getactualminimum.php
     * @param int $field <p>
     * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}.
     * These are integer values between <em>0</em> and
     * <b>IntlCalendar::FIELD_COUNT</b>.
     * </p>
     * @return int|false
     * An {@link https://secure.php.net/manual/en/language.types.integer.php int} representing the minimum value in the field's
     * unit or <b>FALSE</b> on failure.
     *
     * @pure
     */
    public function getActualMinimum(int $field): int|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get array of locales for which there is data
     * @link https://secure.php.net/manual/en/intlcalendar.getavailablelocales.php
     * @return string[] An array of strings, one for which locale.
     */
    public static function getAvailableLocales(): array
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Tell whether a day is a weekday, weekend or a day that has a transition between the two
     * @param int $dayOfWeek <p>
     * One of the constants <b>IntlCalendar::DOW_SUNDAY</b>,
     * <b>IntlCalendar::DOW_MONDAY</b>, ...,
     * <b>IntlCalendar::DOW_SATURDAY</b>.
     * </p>
     * @return int|false
     * Returns one of the constants
     * <b>IntlCalendar::DOW_TYPE_WEEKDAY</b>,
     * <b>IntlCalendar::DOW_TYPE_WEEKEND</b>,
     * <b>IntlCalendar::DOW_TYPE_WEEKEND_OFFSET</b> or
     * <b>IntlCalendar::DOW_TYPE_WEEKEND_CEASE</b> or <b>FALSE</b> on failure.
     *
     * @pure
     */
    public function getDayOfWeekType(int $dayOfWeek): int|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get last error code on the object
     * @link https://secure.php.net/manual/en/intlcalendar.geterrorcode.php
     * @return int|false An ICU error code indicating either success, failure or a warning.
     *
     * @pure
     */
    public function getErrorCode(): int|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get last error message on the object
     * @link https://secure.php.net/manual/en/intlcalendar.geterrormessage.php
     * @return string|false The error message associated with last error that occurred in a function call on this object, or a string indicating the non-existance of an error.
     *
     * @pure
     */
    public function getErrorMessage(): string|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get the first day of the week for the calendar's locale
     * @link https://secure.php.net/manual/en/intlcalendar.getfirstdayofweek.php
     * @return int|false
     * One of the constants <b>IntlCalendar::DOW_SUNDAY</b>,
     * <b>IntlCalendar::DOW_MONDAY</b>, ...,
     * <b>IntlCalendar::DOW_SATURDAY</b> or <b>FALSE</b> on failure.
     *
     * @pure
     */
    public function getFirstDayOfWeek(): int|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get the largest local minimum value for a field
     * @link https://secure.php.net/manual/en/intlcalendar.getgreatestminimum.php
     * @param int $field
     * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
     * values between <em>0</em> and
     * <b>IntlCalendar::FIELD_COUNT</b>.
     * @return int|false
     * An {@link https://secure.php.net/manual/en/language.types.integer.php int} representing a field value, in the field's
     * unit, or <b>FALSE</b> on failure.
     *
     * @pure
     */
    public function getGreatestMinimum(int $field): int|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get set of locale keyword values
     * @param string $keyword <p>
     * The locale keyword for which relevant values are to be queried. Only
     * <em>'calendar'</em> is supported.
     * </p>
     * @param string $locale <p>
     * The locale onto which the keyword/value pair are to be appended.
     * </p>
     * @param bool $onlyCommon
     * <p>
     * Whether to show only the values commonly used for the specified locale.
     * </p>
     * @return Iterator|false An iterator that yields strings with the locale keyword values or <b>FALSE</b> on failure.
     */
    public static function getKeywordValuesForLocale(
        string $keyword,
        string $locale,
        bool $onlyCommon,
    ): IntlIterator|false {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get the smallest local maximum for a field
     * @link https://secure.php.net/manual/en/intlcalendar.getleastmaximum.php
     * @param int $field <p>
     * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
     * values between <em>0</em> and
     * <b>IntlCalendar::FIELD_COUNT</b>.
     * </p>
     * @return int|false
     * An {@link https://secure.php.net/manual/en/language.types.integer.ph int} representing a field value in the field's
     * unit or <b>FALSE</b> on failure.
     *
     * @pure
     */
    public function getLeastMaximum(int $field): int|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get the locale associated with the object
     * @link https://secure.php.net/manual/en/intlcalendar.getlocale.php
     * @param int $type <p>
     * Whether to fetch the actual locale (the locale from which the calendar
     * data originates, with <b>Locale::ACTUAL_LOCALE</b>) or the
     * valid locale, i.e., the most specific locale supported by ICU relatively
     * to the requested locale – see <b>Locale::VALID_LOCALE</b>.
     * From the most general to the most specific, the locales are ordered in
     * this fashion – actual locale, valid locale, requested locale.
     * </p>
     * @return string|false
     *
     * @pure
     */
    public function getLocale(int $type): string|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get the global maximum value for a field
     * @link https://secure.php.net/manual/en/intlcalendar.getmaximum.php
     * @param int $field <p>
     * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
     * values between <em>0</em> and
     * <b>IntlCalendar::FIELD_COUNT</b>.
     * </p>
     * @return int|false
     *
     * @pure
     */
    public function getMaximum(int $field): int|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get minimal number of days the first week in a year or month can have
     * @link https://secure.php.net/manual/en/intlcalendar.getminimaldaysinfirstweek.php
     * @return int|false
     * An {@link https://secure.php.net/manual/en/language.types.integer.php  int} representing a number of days or <b>FALSE</b> on failure.
     *
     * @pure
     */
    public function getMinimalDaysInFirstWeek(): int|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get the global minimum value for a field
     * @link https://secure.php.net/manual/en/intlcalendar.getminimum.php
     * @param int $field <p>
     * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field}. These are integer
     * values between <em>0</em> and
     * <b>IntlCalendar::FIELD_COUNT</b>.
     * </p>
     * @return int|false
     * An int representing a value for the given field in the field's unit or FALSE on failure.
     *
     * @pure
     */
    public function getMinimum(int $field): int|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get number representing the current time
     * @return float A float representing a number of milliseconds since the epoch, not counting leap seconds.
     */
    public static function getNow(): float
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get behavior for handling repeating wall time
     * @link https://secure.php.net/manual/en/intlcalendar.getrepeatedwalltimeoption.php
     * @return int
     * One of the constants <b>IntlCalendar::WALLTIME_FIRST</b> or
     * <b>IntlCalendar::WALLTIME_LAST</b>.
     *
     * @pure
     */
    public function getRepeatedWallTimeOption(): int
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get behavior for handling skipped wall time
     * @link https://secure.php.net/manual/en/intlcalendar.getskippedwalltimeoption.php
     * @return int
     * One of the constants <b>IntlCalendar::WALLTIME_FIRST</b>,
     * <b>IntlCalendar::WALLTIME_LAST</b> or
     * <b>IntlCalendar::WALLTIME_NEXT_VALID</b>.
     *
     * @pure
     */
    public function getSkippedWallTimeOption(): int
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get time currently represented by the object
     * @return float|false
     * A {@link https://secure.php.net/manual/en/language.types.float.php float} representing the number of milliseconds elapsed since the
     * reference time (1 Jan 1970 00:00:00 UTC).
     *
     * @pure
     */
    public function getTime(): float|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get the object's timezone
     * @link https://secure.php.net/manual/en/intlcalendar.gettimezone.php
     * @return IntlTimeZone|false
     * An {@link https://secure.php.net/manual/en/class.intltimezone.php IntlTimeZone} object corresponding to the one used
     * internally in this object.
     *
     * @pure
     */
    public function getTimeZone(): IntlTimeZone|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get the calendar type
     * @link https://secure.php.net/manual/en/intlcalendar.gettype.php
     * @return string
     * A {@link https://secure.php.net/manual/en/language.types.string.php string} representing the calendar type, such as
     * <em>'gregorian'</em>, <em>'islamic'</em>, etc.
     *
     * @pure
     */
    public function getType(): string
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get time of the day at which weekend begins or ends
     * @link https://secure.php.net/manual/en/intlcalendar.getweekendtransition.php
     * @param string $dayOfWeek <p>
     * One of the constants <b>IntlCalendar::DOW_SUNDAY</b>,
     * <b>IntlCalendar::DOW_MONDAY</b>, ...,
     * <b>IntlCalendar::DOW_SATURDAY</b>.
     * </p>
     * @return int|false
     * The number of milliseconds into the day at which the the weekend begins or
     * ends or <b>FALSE</b> on failure.
     *
     * @pure
     */
    public function getWeekendTransition(int $dayOfWeek): int|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Whether the object's time is in Daylight Savings Time
     * @link https://secure.php.net/manual/en/intlcalendar.indaylighttime.php
     * @return bool
     * Returns <b>TRUE</b> if the date is in Daylight Savings Time, <b>FALSE</b> otherwise.
     * The value <b>FALSE</b> may also be returned on failure, for instance after
     * specifying invalid field values on non-lenient mode; use {@link https://secure.php.net/manual/en/intl.configuration.php#ini.intl.use-exceptions exceptions} or query
     * {@link https://secure.php.net/manual/en/function.intl-get-error-code.php intl_get_error_code()} to disambiguate.
     *
     * @pure
     */
    public function inDaylightTime(): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Whether another calendar is equal but for a different time
     * @link https://secure.php.net/manual/en/intlcalendar.isequivalentto.php
     * @param IntlCalendar $other The other calendar against which the comparison is to be made.
     * @return bool
     * Assuming there are no argument errors, returns <b>TRUE</b> iif the calendars are equivalent except possibly for their set time.
     *
     * @pure
     */
    public function isEquivalentTo(IntlCalendar $other): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Whether date/time interpretation is in lenient mode
     * @link https://secure.php.net/manual/en/intlcalendar.islenient.php
     * @return bool
     * A {@link https://secure.php.net/manual/en/language.types.boolean.php bool} representing whether the calendar is set to lenient mode.
     *
     * @pure
     */
    public function isLenient(): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Whether a certain date/time is in the weekend
     * @link https://secure.php.net/manual/en/intlcalendar.isweekend.php
     * @param float|null $timestamp [optional] <p>
     * An optional timestamp representing the number of milliseconds since the
     * epoch, excluding leap seconds. If <b>NULL</b>, this object's current time is
     * used instead.
     * </p>
     * @return bool
     * <p> A {@link https://secure.php.net/manual/en/language.types.boolean.php bool} indicating whether the given or this object's time occurs
     * in a weekend.
     * </p>
     * <p>
     * The value <b>FALSE</b> may also be returned on failure, for instance after giving
     * a date out of bounds on non-lenient mode; use {@link https://secure.php.net/manual/en/intl.configuration.php#ini.intl.use-exceptions exceptions} or query
     * {@link https://secure.php.net/manual/en/function.intl-get-error-code.php intl_get_error_code()} to disambiguate.</p>
     *
     * @pure
     */
    public function isWeekend(float|null $timestamp = null): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Add value to field without carrying into more significant fields
     * @link https://secure.php.net/manual/en/intlcalendar.roll.php
     * @param int $field
     * <p>One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time
     * {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
     * values between <em>0</em> and
     * <b>IntlCalendar::FIELD_COUNT</b>.
     * </p>
     * @param mixed $value <p>
     * The (signed) amount to add to the field, <b>TRUE</b> for rolling up (adding
     * <em>1</em>), or <b>FALSE</b> for rolling down (subtracting
     * <em>1</em>).
     * </p>
     * @return bool Returns <b>TRUE</b> on success or <b>FALSE</b> on failure.
     */
    public function roll(int $field, $value): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Whether a field is set
     * @link https://secure.php.net/manual/en/intlcalendar.isset.php
     * @param int $field <p>
     * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time
     * {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}.
     * These are integer values between <em>0</em> and
     * <b>IntlCalendar::FIELD_COUNT</b>.
     * </p>
     * @return bool Assuming there are no argument errors, returns <b>TRUE</b> iif the field is set.
     */
    public function PS_UNRESERVE_PREFIX_isSet(int $field): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Set a time field or several common fields at once
     * @link https://secure.php.net/manual/en/intlcalendar.set.php
     * @param int $year <p>
     * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
     * values between <em>0</em> and
     * <b>IntlCalendar::FIELD_COUNT</b>.
     * </p>
     * @param int $month <p>
     * The new value for <b>IntlCalendar::FIELD_MONTH</b>.
     * </p>
     * @param int $dayOfMonth [optional] <p>
     * The new value for <b>IntlCalendar::FIELD_DAY_OF_MONTH</b>.
     * The month sequence is zero-based, i.e., January is represented by 0,
     * February by 1, ..., December is 11 and Undecember (if the calendar has
     * it) is 12.
     * </p>
     * @param int $hour [optional]
     * <p>
     * The new value for <b>IntlCalendar::FIELD_HOUR_OF_DAY</b>.
     * </p>
     * @param int $minute [optional]
     * <p>
     * The new value for <b>IntlCalendar::FIELD_MINUTE</b>.
     * </p>
     * @param int $second [optional] <p>
     * The new value for <b>IntlCalendar::FIELD_SECOND</b>.
     * </p>
     * @return bool Returns <b>TRUE</b> on success and <b>FALSE</b> on failure.
     */
    public function set($year, $month, $dayOfMonth = null, $hour = null, $minute = null, $second = null)
    {
    }

    /**
     * (PHP 5 >= 5.5.0 PECL intl >= 3.0.0a1)<br/>
     * Set a time field or several common fields at once
     * @link https://secure.php.net/manual/en/intlcalendar.set.php
     * @param int $field One of the IntlCalendar date/time field constants. These are integer values between 0 and IntlCalendar::FIELD_COUNT.
     * @param int $value The new value of the given field.
     * @return bool Returns <b>TRUE</b> on success and <b>FALSE</b> on failure.
     * @since 5.5
     */
    public function set($field, $value)
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Set the day on which the week is deemed to start
     * @link https://secure.php.net/manual/en/intlcalendar.setfirstdayofweek.php
     * @param int $dayOfWeek <p>
     * One of the constants <b>IntlCalendar::DOW_SUNDAY</b>,
     * <b>IntlCalendar::DOW_MONDAY</b>, ...,
     * <b>IntlCalendar::DOW_SATURDAY</b>.
     * </p>
     * @return bool Returns TRUE on success. Failure can only happen due to invalid parameters.
     */
    public function setFirstDayOfWeek(int $dayOfWeek): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Set whether date/time interpretation is to be lenient
     * @link https://secure.php.net/manual/en/intlcalendar.setlenient.php
     * @param bool $lenient <p>
     * Use <b>TRUE</b> to activate the lenient mode; <b>FALSE</b> otherwise.
     * </p>
     * @return bool Returns <b>TRUE</b> on success. Failure can only happen due to invalid parameters.
     */
    public function setLenient(bool $lenient): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Set behavior for handling repeating wall times at negative timezone offset transitions
     * @link https://secure.php.net/manual/en/intlcalendar.setrepeatedwalltimeoption.php
     * @param int $option <p>
     * One of the constants <b>IntlCalendar::WALLTIME_FIRST</b> or
     * <b>IntlCalendar::WALLTIME_LAST</b>.
     * </p>
     * @return bool
     * Returns <b>TRUE</b> on success. Failure can only happen due to invalid parameters.
     */
    public function setRepeatedWallTimeOption(int $option): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Set behavior for handling skipped wall times at positive timezone offset transitions
     * @link https://secure.php.net/manual/en/intlcalendar.setskippedwalltimeoption.php
     * @param int $option <p>
     * One of the constants <b>IntlCalendar::WALLTIME_FIRST</b>,
     * <b>IntlCalendar::WALLTIME_LAST</b> or
     * <b>IntlCalendar::WALLTIME_NEXT_VALID</b>.
     * </p>
     * @return bool
     * <p>
     * Returns <b>TRUE</b> on success. Failure can only happen due to invalid parameters.
     * </p>
     */
    public function setSkippedWallTimeOption(int $option): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Set the calendar time in milliseconds since the epoch
     * @link https://secure.php.net/manual/en/intlcalendar.settime.php
     * @param float $timestamp <p>
     * An instant represented by the number of number of milliseconds between
     * such instant and the epoch, ignoring leap seconds.
     * </p>
     * @return bool
     * Returns <b>TRUE</b> on success and <b>FALSE</b> on failure.
     */
    public function setTime(float $timestamp): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Set the timezone used by this calendar
     * @link https://secure.php.net/manual/en/intlcalendar.settimezone.php
     * @param mixed $timezone <p>
     * The new timezone to be used by this calendar. It can be specified in the
     * following ways:
     *
     * </p><ul>
     * <li>
     * <p>
     * <b>NULL</b>, in which case the default timezone will be used, as specified in
     * the ini setting {@link https://secure.php.net/manual/en/datetime.configuration.php#ini.date.timezone date.timezone} or
     * through the function  {@link https://secure.php.net/manual/en/function.date-default-timezone-set.php date_default_timezone_set()} and as
     * returned by  {@link https://secure.php.net/manual/en/function.date-default-timezone-get.php date_default_timezone_get()}.
     * </p>
     * </li>
     * <li>
     * <p>
     * An {@link https://secure.php.net/manual/en/class.intltimezone.php IntlTimeZone}, which will be used directly.
     * </p>
     * </li>
     * <li>
     * <p>
     * A {@link https://secure.php.net/manual/en/class.datetimezone.php DateTimeZone}. Its identifier will be extracted
     * and an ICU timezone object will be created; the timezone will be backed
     * by ICU's database, not PHP's.
     * </p>
     * </li>
     * <li>
     * <p>
     * A {@link https://secure.php.net/manual/en/language.types.string.php string}, which should be a valid ICU timezone identifier.
     * See  b>IntlTimeZone::createTimeZoneIDEnumeration()</b>. Raw
     * offsets such as <em>"GMT+08:30"</em> are also accepted.
     * </p>
     * </li>
     * </ul>
     * @return bool Returns <b>TRUE</b> on success and <b>FALSE</b> on failure.
     */
    public function setTimeZone($timezone): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a2)<br/>
     * Convert an IntlCalendar into a DateTime object
     * @link https://secure.php.net/manual/en/intlcalendar.todatetime.php
     * @return DateTime|false
     * A {@link https://secure.php.net/manual/en/class.datetime.php DateTime} object with the same timezone as this
     * object (though using PHP's database instead of ICU's) and the same time,
     * except for the smaller precision (second precision instead of millisecond).
     * Returns <b>FALSE</b> on failure.
     *
     * @pure
     */
    public function toDateTime(): DateTime|false
    {
    }

    /**
     * @link https://www.php.net/manual/en/intlcalendar.setminimaldaysinfirstweek.php
     * @param int $days
     * @return bool
     */
    public function setMinimalDaysInFirstWeek(int $days): bool
    {
    }

    /**
     * @since 8.3
     */
    public function setDate(int $year, int $month, int $dayOfMonth): void
    {
    }

    /**
     * @since 8.3
     */
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

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get the number of IDs in the equivalency group that includes the given ID
     * @link https://secure.php.net/manual/en/intltimezone.countequivalentids.php
     * @param string $timezoneId
     * @return int|false number of IDs or <b>FALSE</b> on failure
     */
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

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get an enumeration over system time zone IDs with the given filter conditions
     * @link https://secure.php.net/manual/en/intltimezone.createtimezoneidenumeration.php
     * @param int $type
     * @param string|null $region [optional]
     * @param int $rawOffset [optional]
     * @return IntlIterator|false an iterator or <b>FALSE</b> on failure
     */
    public static function createTimeZoneIDEnumeration(
        int $type,
        string|null $region = null,
        null|int $rawOffset = null,
    ): IntlIterator|false {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Create a timezone object from DateTimeZone
     * @link https://secure.php.net/manual/en/intltimezone.fromdatetimezone.php
     * @param DateTimeZone $timezone
     * @return IntlTimeZone|null a timezone object or <b>NULL</b> on failure
     */
    public static function fromDateTimeZone(DateTimeZone $timezone): null|IntlTimeZone
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get the canonical system timezone ID or the normalized custom time zone ID for the given time zone ID
     * @link https://secure.php.net/manual/en/intltimezone.getcanonicalid.php
     * @param string $timezoneId
     * @param bool &$isSystemId [optional]
     * @return string|false the timezone ID or <b>FALSE</b> on failure
     */
    public static function getCanonicalID(string $timezoneId, &$isSystemId): string|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get a name of this time zone suitable for presentation to the user
     * @param bool $dst [optional]
     * @param int $style [optional]
     * @param string $locale [optional]
     * @return string|false the timezone name or <b>FALSE</b> on failure
     *
     * @pure
     */
    public function getDisplayName(bool $dst = false, int $style = 2, string|null $locale): string|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get the amount of time to be added to local standard time to get local wall clock time
     * @link https://secure.php.net/manual/en/intltimezone.getequivalentid.php
     * @return int
     *
     * @pure
     */
    public function getDSTSavings(): int
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get an ID in the equivalency group that includes the given ID
     * @link https://secure.php.net/manual/en/intltimezone.getequivalentid.php
     * @param string $timezoneId
     * @param int $offset
     * @return string|false the time zone ID or <b>FALSE</b> on failure
     */
    public static function getEquivalentID(string $timezoneId, int $offset): string|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get last error code on the object
     * @link https://secure.php.net/manual/en/intltimezone.geterrorcode.php
     * @return int|false
     *
     * @pure
     */
    public function getErrorCode(): int|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get last error message on the object
     * @link https://secure.php.net/manual/en/intltimezone.geterrormessage.php
     * @return string|false
     *
     * @pure
     */
    public function getErrorMessage(): string|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Create GMT (UTC) timezone
     * @link https://secure.php.net/manual/en/intltimezone.getgmt.php
     * @return IntlTimeZone
     */
    public static function getGMT(): IntlTimeZone
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get timezone ID
     * @return string|false
     *
     * @pure
     */
    public function getID(): string|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get the time zone raw and GMT offset for the given moment in time
     * @link https://secure.php.net/manual/en/intltimezone.getoffset.php
     * @param float $timestamp
     *   moment in time for which to return offsets, in units of milliseconds from
     *   January 1, 1970 0:00 GMT, either GMT time or local wall time, depending on
     *   `local'.
     * @param bool $local
     *   if true, `date' is local wall time; otherwise it is in GMT time.
     * @param int &$rawOffset
     *   output parameter to receive the raw offset, that is, the offset not
     *   including DST adjustments
     * @param int &$dstOffset
     *   output parameter to receive the DST offset, that is, the offset to be added
     *   to `rawOffset' to obtain the total offset between local and GMT time. If
     *   DST is not in effect, this value is zero; otherwise it is a positive value,
     *   typically one hour.
     * @return bool boolean indication of success
     */
    public function getOffset(float $timestamp, bool $local, &$rawOffset, &$dstOffset): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get the raw GMT offset (before taking daylight savings time into account
     * @link https://secure.php.net/manual/en/intltimezone.getrawoffset.php
     * @return int
     *
     * @pure
     */
    public function getRawOffset(): int
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get the region code associated with the given system time zone ID
     * @link https://secure.php.net/manual/en/intltimezone.getregion.php
     * @param string $timezoneId
     * @return string|false region or <b>FALSE</b> on failure
     */
    public static function getRegion(string $timezoneId): string|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Get the timezone data version currently used by ICU
     * @link https://secure.php.net/manual/en/intltimezone.gettzdataversion.php
     * @return string|false
     */
    public static function getTZDataVersion(): string|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get the "unknown" time zone
     * @link https://secure.php.net/manual/en/intltimezone.getunknown.php
     * @return IntlTimeZone
     */
    public static function getUnknown(): IntlTimeZone
    {
    }

    /**
     * (PHP 7 &gt;=7.1.0)<br/>
     * Translates a system timezone (e.g. "America/Los_Angeles") into a Windows
     * timezone (e.g. "Pacific Standard Time").
     * @link https://secure.php.net/manual/en/intltimezone.getwindowsid.php
     * @param string $timezoneId
     * @return string|false the Windows timezone or <b>FALSE</b> on failure
     * @since 7.1
     */
    public static function getWindowsID(string $timezoneId): string|false
    {
    }

    /**
     * @link https://www.php.net/manual/en/intltimezone.getidforwindowsid.php
     * @param string $timezoneId
     * @param string|null $region
     * @return string|false the Windows timezone or <b>FALSE</b> on failure
     * @since 7.1
     */
    public static function getIDForWindowsID(string $timezoneId, null|string $region = null): string|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Check if this zone has the same rules and offset as another zone
     * @link https://secure.php.net/manual/en/intltimezone.hassamerules.php
     * @param IntlTimeZone $other
     * @return bool
     *
     * @pure
     */
    public function hasSameRules(IntlTimeZone $other): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Convert to DateTimeZone object
     * @link https://secure.php.net/manual/en/intltimezone.todatetimezone.php
     * @return DateTimeZone|false the DateTimeZone object or <b>FALSE</b> on failure
     *
     * @pure
     */
    public function toDateTimeZone(): DateTimeZone|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
     * Check if this time zone uses daylight savings time
     * @link https://secure.php.net/manual/en/intltimezone.usedaylighttime.php
     * @return bool
     */
    public function useDaylightTime(): bool
    {
    }

    /**
     * @since 8.4
     */
    public static function getIanaID(string $timezoneId): string|false
    {
    }
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Create a collator
 * @link https://php.net/manual/en/collator.create.php
 * @param string $locale <p>
 * The locale containing the required collation rules. Special values for
 * locales can be passed in - if null is passed for the locale, the
 * default locale collation rules will be used. If empty string ("") or
 * "root" are passed, UCA rules will be used.
 * </p>
 * @return Collator|null Return new instance of <b>Collator</b> object, or <b>NULL</b>
 * on error.
 * @pure
 */
function collator_create(string $locale): null|Collator
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Compare two Unicode strings
 * @link https://php.net/manual/en/collator.compare.php
 * @param Collator $object
 * @param string $string1 <p>
 * The first string to compare.
 * </p>
 * @param string $string2 <p>
 * The second string to compare.
 * </p>
 * @return int|false Return comparison result:</p>
 * <p>
 * <p>
 * 1 if <i>string1</i> is greater than
 * <i>string2</i> ;
 * </p>
 * <p>
 * 0 if <i>string1</i> is equal to
 * <i>string2</i>;
 * </p>
 * <p>
 * -1 if <i>string1</i> is less than
 * <i>string2</i> .
 * </p>
 * On error
 * boolean
 * <b>FALSE</b>
 * is returned.
 * @pure
 */
function collator_compare(Collator $object, string $string1, string $string2): int|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get collation attribute value
 * @link https://php.net/manual/en/collator.getattribute.php
 * @param Collator $object
 * @param int $attribute <p>
 * Attribute to get value for.
 * </p>
 * @return int|false Attribute value, or boolean <b>FALSE</b> on error.
 * @pure
 */
function collator_get_attribute(Collator $object, int $attribute): int|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Set collation attribute
 * @link https://php.net/manual/en/collator.setattribute.php
 * @param Collator $object
 * @param int $attribute <p>Attribute.</p>
 * @param int $value <p>
 * Attribute value.
 * </p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function collator_set_attribute(Collator $object, int $attribute, int $value): bool
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get current collation strength
 * @link https://php.net/manual/en/collator.getstrength.php
 * @param Collator $object
 * @return int current collation strength
 * @pure
 */
function collator_get_strength(Collator $object): int
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Set collation strength
 * @link https://php.net/manual/en/collator.setstrength.php
 * @param Collator $object
 * @param int $strength <p>Strength to set.</p>
 * <p>
 * Possible values are:
 * <b>Collator::PRIMARY</b>
 * </p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function collator_set_strength(Collator $object, int $strength): bool
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Sort array using specified collator
 * @link https://php.net/manual/en/collator.sort.php
 * @param Collator $object
 * @param string[] &$array <p>
 * Array of strings to sort.
 * </p>
 * @param int $flags <p>
 * Optional sorting type, one of the following:
 * </p>
 * <p>
 * <b>Collator::SORT_REGULAR</b>
 * - compare items normally (don't change types)
 * </p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function collator_sort(Collator $object, array &$array, int $flags = 0): bool
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Sort array using specified collator and sort keys
 * @link https://php.net/manual/en/collator.sortwithsortkeys.php
 * @param Collator $object
 * @param string[] &$array <p>Array of strings to sort</p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function collator_sort_with_sort_keys(Collator $object, array &$array): bool
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Sort array maintaining index association
 * @link https://php.net/manual/en/collator.asort.php
 * @param Collator $object
 * @param string[] &$array <p>Array of strings to sort.</p>
 * @param int $flags <p>
 * Optional sorting type, one of the following:
 * <b>Collator::SORT_REGULAR</b>
 * - compare items normally (don't change types)
 * </p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function collator_asort(Collator $object, array &$array, int $flags = 0): bool
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get the locale name of the collator
 * @link https://php.net/manual/en/collator.getlocale.php
 * @param Collator $object
 * @param int $type <p>
 * You can choose between valid and actual locale (
 * <b>Locale::VALID_LOCALE</b> and
 * <b>Locale::ACTUAL_LOCALE</b>,
 * respectively). The default is the actual locale.
 * </p>
 * @return string|false Real locale name from which the collation data comes. If the collator was
 * instantiated from rules or an error occurred, returns
 * boolean <b>FALSE</b>.
 * @pure
 */
function collator_get_locale(Collator $object, int $type): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get collator's last error code
 * @link https://php.net/manual/en/collator.geterrorcode.php
 * @param Collator $object
 * @return int|false Error code returned by the last Collator API function call.
 */
function collator_get_error_code(Collator $object): int|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get text for collator's last error code
 * @link https://php.net/manual/en/collator.geterrormessage.php
 * @param Collator $object
 * @return string|false Description of an error occurred in the last Collator API function call.
 * @pure
 */
function collator_get_error_message(Collator $object): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.2, PHP 7, PECL intl &gt;= 1.0.3)<br/>
 * Get sorting key for a string
 * @link https://php.net/manual/en/collator.getsortkey.php
 * @param Collator $object
 * @param string $string <p>
 * The string to produce the key from.
 * </p>
 * @return string|false the collation key for the string. Collation keys can be compared directly instead of strings.
 * @pure
 */
function collator_get_sort_key(Collator $object, string $string): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Create a number formatter
 * @link https://php.net/manual/en/numberformatter.create.php
 * @param string $locale <p>
 * Locale in which the number would be formatted (locale name, e.g. en_CA).
 * </p>
 * @param int $style <p>
 * Style of the formatting, one of the
 * format style constants. If
 * <b>NumberFormatter::PATTERN_DECIMAL</b>
 * or <b>NumberFormatter::PATTERN_RULEBASED</b>
 * is passed then the number format is opened using the given pattern,
 * which must conform to the syntax described in
 * ICU DecimalFormat
 * documentation or
 * ICU RuleBasedNumberFormat
 * documentation, respectively.
 * </p>
 * @param string|null $pattern [optional] <p>
 * Pattern string if the chosen style requires a pattern.
 * </p>
 * @return NumberFormatter|null <b>NumberFormatter</b> object or <b>NULL</b> on error.
 * @pure
 */
function numfmt_create(string $locale, int $style, string|null $pattern = null): null|NumberFormatter
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Format a number
 * @link https://php.net/manual/en/numberformatter.format.php
 * @param NumberFormatter $formatter
 * @param int|float $num <p>
 * The value to format. Can be integer or float,
 * other values will be converted to a numeric value.
 * </p>
 * @param int $type <p>
 * The
 * formatting type to use.
 * </p>
 * @return string|false the string containing formatted value, or <b>FALSE</b> on error.
 * @pure
 */
function numfmt_format(NumberFormatter $formatter, int|float $num, int $type = 0): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Parse a number
 * @link https://php.net/manual/en/numberformatter.parse.php
 * @param NumberFormatter $formatter
 * @param string $string
 * @param int $type [optional] <p>
 * The
 * formatting type to use. By default,
 * <b>NumberFormatter::TYPE_DOUBLE</b> is used.
 * </p>
 * @param int &$offset [optional] <p>
 * Offset in the string at which to begin parsing. On return, this value
 * will hold the offset at which parsing ended.
 * </p>
 * @return int|float|false The value of the parsed number or <b>FALSE</b> on error.
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
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Format a currency value
 * @link https://php.net/manual/en/numberformatter.formatcurrency.php
 * @param NumberFormatter $formatter
 * @param float $amount <p>
 * The numeric currency value.
 * </p>
 * @param string $currency <p>
 * The 3-letter ISO 4217 currency code indicating the currency to use.
 * </p>
 * @return string|false String representing the formatted currency value.
 * @pure
 */
function numfmt_format_currency(NumberFormatter $formatter, float $amount, string $currency): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Parse a currency number
 * @link https://php.net/manual/en/numberformatter.parsecurrency.php
 * @param NumberFormatter $formatter
 * @param string $string
 * @param string &$currency <p>
 * Parameter to receive the currency name (3-letter ISO 4217 currency
 * code).
 * </p>
 * @param int &$offset [optional] <p>
 * Offset in the string at which to begin parsing. On return, this value
 * will hold the offset at which parsing ended.
 * </p>
 * @return float|false The parsed numeric value or <b>FALSE</b> on error.
 */
function numfmt_parse_currency(NumberFormatter $formatter, string $string, &$currency, &$offset = null): float|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Set an attribute
 * @link https://php.net/manual/en/numberformatter.setattribute.php
 * @param NumberFormatter $formatter
 * @param int $attribute <p>
 * Attribute specifier - one of the
 * numeric attribute constants.
 * </p>
 * @param int|float $value <p>
 * The attribute value.
 * </p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function numfmt_set_attribute(NumberFormatter $formatter, int $attribute, int|float $value): bool
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get an attribute
 * @link https://php.net/manual/en/numberformatter.getattribute.php
 * @param NumberFormatter $formatter
 * @param int $attribute <p>
 * Attribute specifier - one of the
 * numeric attribute constants.
 * </p>
 * @return int|float|false Return attribute value on success, or <b>FALSE</b> on error.
 * @pure
 */
function numfmt_get_attribute(NumberFormatter $formatter, int $attribute): int|float|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Set a text attribute
 * @link https://php.net/manual/en/numberformatter.settextattribute.php
 * @param NumberFormatter $formatter
 * @param int $attribute <p>
 * Attribute specifier - one of the
 * text attribute
 * constants.
 * </p>
 * @param string $value <p>
 * Text for the attribute value.
 * </p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function numfmt_set_text_attribute(NumberFormatter $formatter, int $attribute, string $value): bool
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get a text attribute
 * @link https://php.net/manual/en/numberformatter.gettextattribute.php
 * @param NumberFormatter $formatter
 * @param int $attribute <p>
 * Attribute specifier - one of the
 * text attribute constants.
 * </p>
 * @return string|false Return attribute value on success, or <b>FALSE</b> on error.
 * @pure
 */
function numfmt_get_text_attribute(NumberFormatter $formatter, int $attribute): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Set a symbol value
 * @link https://php.net/manual/en/numberformatter.setsymbol.php
 * @param NumberFormatter $formatter
 * @param int $symbol <p>
 * Symbol specifier, one of the
 * format symbol constants.
 * </p>
 * @param string $value <p>
 * Text for the symbol.
 * </p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function numfmt_set_symbol(NumberFormatter $formatter, int $symbol, string $value): bool
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get a symbol value
 * @link https://php.net/manual/en/numberformatter.getsymbol.php
 * @param NumberFormatter $formatter
 * @param int $symbol <p>
 * Symbol specifier, one of the
 * format symbol constants.
 * </p>
 * @return string|false The symbol string or <b>FALSE</b> on error.
 * @pure
 */
function numfmt_get_symbol(NumberFormatter $formatter, int $symbol): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Set formatter pattern
 * @link https://php.net/manual/en/numberformatter.setpattern.php
 * @param NumberFormatter $formatter
 * @param string $pattern <p>
 * Pattern in syntax described in
 * ICU DecimalFormat
 * documentation.
 * </p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function numfmt_set_pattern(NumberFormatter $formatter, string $pattern): bool
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get formatter pattern
 * @link https://php.net/manual/en/numberformatter.getpattern.php
 * @param NumberFormatter $formatter
 * @return string|false Pattern string that is used by the formatter, or <b>FALSE</b> if an error happens.
 * @pure
 */
function numfmt_get_pattern(NumberFormatter $formatter): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get formatter locale
 * @link https://php.net/manual/en/numberformatter.getlocale.php
 * @param NumberFormatter $formatter
 * @param int $type <p>
 * You can choose between valid and actual locale (
 * <b>Locale::VALID_LOCALE</b>,
 * <b>Locale::ACTUAL_LOCALE</b>,
 * respectively). The default is the actual locale.
 * </p>
 * @return string|false The locale name used to create the formatter.
 * @pure
 */
function numfmt_get_locale(NumberFormatter $formatter, int $type = 0): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get formatter's last error code.
 * @link https://php.net/manual/en/numberformatter.geterrorcode.php
 * @param NumberFormatter $formatter
 * @return int error code from last formatter call.
 */
function numfmt_get_error_code(NumberFormatter $formatter): int
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get formatter's last error message.
 * @link https://php.net/manual/en/numberformatter.geterrormessage.php
 * @param NumberFormatter $formatter
 * @return string error message from last formatter call.
 */
function numfmt_get_error_message(NumberFormatter $formatter): string
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Normalizes the input provided and returns the normalized string
 * @link https://php.net/manual/en/normalizer.normalize.php
 * @param string $string <p>The input string to normalize</p>
 * @param int $form [optional] <p>One of the normalization forms.</p>
 * @return string|false The normalized string or <b>FALSE</b> if an error occurred.
 * @pure
 */
function normalizer_normalize(string $string, int $form = Normalizer::FORM_C): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Checks if the provided string is already in the specified normalization
 * form.
 * @link https://php.net/manual/en/normalizer.isnormalized.php
 * @param string $string <p>The input string to normalize</p>
 * @param int $form [optional] <p>
 * One of the normalization forms.
 * </p>
 * @return bool <b>TRUE</b> if normalized, <b>FALSE</b> otherwise or if there an error
 * @pure
 */
function normalizer_is_normalized(string $string, int $form = Normalizer::FORM_C): bool
{
}

/**
 * Gets the default locale value from the intl global 'default_locale'
 * @link https://php.net/manual/en/function.locale-get-default.php
 * @return string a string with the current Locale.
 * @pure
 */
function locale_get_default(): string
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Set the default runtime Locale
 * @link https://php.net/manual/en/function.locale-set-default.php
 * @param string $locale <p>
 * The new Locale name. A comprehensive list of the supported locales is
 * available at .
 * </p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function locale_set_default(string $locale): bool
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Gets the primary language for the input locale
 * @link https://php.net/manual/en/locale.getprimarylanguage.php
 * @param string $locale <p>
 * The locale to extract the primary language code from
 * </p>
 * @return string|null The language code associated with the language or <b>NULL</b> in case of error.
 * @pure
 */
function locale_get_primary_language(string $locale): null|string
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Gets the script for the input locale
 * @link https://php.net/manual/en/locale.getscript.php
 * @param string $locale <p>
 * The locale to extract the script code from
 * </p>
 * @return string|null The script subtag for the locale or <b>NULL</b> if not present
 * @pure
 */
function locale_get_script(string $locale): null|string
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Gets the region for the input locale
 * @link https://php.net/manual/en/locale.getregion.php
 * @param string $locale <p>
 * The locale to extract the region code from
 * </p>
 * @return string|null The region subtag for the locale or <b>NULL</b> if not present
 * @pure
 */
function locale_get_region(string $locale): null|string
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Gets the keywords for the input locale
 * @link https://php.net/manual/en/locale.getkeywords.php
 * @param string $locale <p>
 * The locale to extract the keywords from
 * </p>
 * @return array|false|null Associative array containing the keyword-value pairs for this locale
 * @pure
 */
function locale_get_keywords(string $locale): array|false|null
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Returns an appropriately localized display name for script of the input locale
 * @link https://php.net/manual/en/locale.getdisplayscript.php
 * @param string $locale <p>
 * The locale to return a display script for
 * </p>
 * @param string|null $displayLocale <p>
 * Optional format locale to use to display the script name
 * </p>
 * @return string|false Display name of the script for the $locale in the format appropriate for
 * $in_locale.
 * @pure
 */
function locale_get_display_script(string $locale, null|string $displayLocale = null): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Returns an appropriately localized display name for region of the input locale
 * @link https://php.net/manual/en/locale.getdisplayregion.php
 * @param string $locale <p>
 * The locale to return a display region for.
 * </p>
 * @param string|null $displayLocale <p>
 * Optional format locale to use to display the region name
 * </p>
 * @return string|false display name of the region for the $locale in the format appropriate for
 * $in_locale.
 * @pure
 */
function locale_get_display_region(string $locale, null|string $displayLocale = null): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Returns an appropriately localized display name for the input locale
 * @link https://php.net/manual/en/locale.getdisplayname.php
 * @param string $locale <p>
 * The locale to return a display name for.
 * </p>
 * @param string|null $displayLocale <p>optional format locale</p>
 * @return string|false Display name of the locale in the format appropriate for $in_locale.
 * @pure
 */
function locale_get_display_name(string $locale, null|string $displayLocale = null): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Returns an appropriately localized display name for language of the inputlocale
 * @link https://php.net/manual/en/locale.getdisplaylanguage.php
 * @param string $locale <p>
 * The locale to return a display language for
 * </p>
 * @param string|null $displayLocale <p>
 * Optional format locale to use to display the language name
 * </p>
 * @return string|false display name of the language for the $locale in the format appropriate for
 * $in_locale.
 * @pure
 */
function locale_get_display_language(string $locale, null|string $displayLocale = null): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Returns an appropriately localized display name for variants of the input locale
 * @link https://php.net/manual/en/locale.getdisplayvariant.php
 * @param string $locale <p>
 * The locale to return a display variant for
 * </p>
 * @param string|null $displayLocale <p>
 * Optional format locale to use to display the variant name
 * </p>
 * @return string|false Display name of the variant for the $locale in the format appropriate for
 * $in_locale.
 * @pure
 */
function locale_get_display_variant(string $locale, null|string $displayLocale = null): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Returns a correctly ordered and delimited locale ID
 * @link https://php.net/manual/en/locale.composelocale.php
 * @param string[] $subtags <p>
 * an array containing a list of key-value pairs, where the keys identify
 * the particular locale ID subtags, and the values are the associated
 * subtag values.
 * <p>
 * The 'variant' and 'private' subtags can take maximum 15 values
 * whereas 'extlang' can take maximum 3 values.e.g. Variants are allowed
 * with the suffix ranging from 0-14. Hence the keys for the input array
 * can be variant0, variant1, ...,variant14. In the returned locale id,
 * the subtag is ordered by suffix resulting in variant0 followed by
 * variant1 followed by variant2 and so on.
 * </p>
 * <p>
 * The 'variant', 'private' and 'extlang' multiple values can be specified both
 * as array under specific key (e.g. 'variant') and as multiple numbered keys
 * (e.g. 'variant0', 'variant1', etc.).
 * </p>
 * </p>
 * @return string|false The corresponding locale identifier.
 * @pure
 */
function locale_compose(array $subtags): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Returns a key-value array of locale ID subtag elements.
 * @link https://php.net/manual/en/locale.parselocale.php
 * @param string $locale <p>
 * The locale to extract the subtag array from. Note: The 'variant' and
 * 'private' subtags can take maximum 15 values whereas 'extlang' can take
 * maximum 3 values.
 * </p>
 * @return string[]|null an array containing a list of key-value pairs, where the keys
 * identify the particular locale ID subtags, and the values are the
 * associated subtag values. The array will be ordered as the locale id
 * subtags e.g. in the locale id if variants are '-varX-varY-varZ' then the
 * returned array will have variant0=&gt;varX , variant1=&gt;varY ,
 * variant2=&gt;varZ
 * @pure
 */
function locale_parse(string $locale): null|array
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Gets the variants for the input locale
 * @link https://php.net/manual/en/locale.getallvariants.php
 * @param string $locale <p>
 * The locale to extract the variants from
 * </p>
 * @return array|null The array containing the list of all variants subtag for the locale
 * or <b>NULL</b> if not present
 * @pure
 */
function locale_get_all_variants(string $locale): null|array
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Checks if a language tag filter matches with locale
 * @link https://php.net/manual/en/locale.filtermatches.php
 * @param string $languageTag <p>
 * The language tag to check
 * </p>
 * @param string $locale <p>
 * The language range to check against
 * </p>
 * @param bool $canonicalize <p>
 * If true, the arguments will be converted to canonical form before
 * matching.
 * </p>
 * @return bool|null <b>TRUE</b> if $locale matches $langtag <b>FALSE</b> otherwise.
 * @pure
 */
function locale_filter_matches(string $languageTag, string $locale, bool $canonicalize = false): null|bool
{
}

/**
 * Canonicalize the locale string
 * @param string $locale
 *
 * @return null|string
 * @pure
 */
function locale_canonicalize(string $locale): null|string
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Searches the language tag list for the best match to the language
 * @link https://php.net/manual/en/locale.lookup.php
 * @param string[] $languageTag <p>
 * An array containing a list of language tags to compare to
 * <i>locale</i>. Maximum 100 items allowed.
 * </p>
 * @param string $locale <p>
 * The locale to use as the language range when matching.
 * </p>
 * @param bool $canonicalize <p>
 * If true, the arguments will be converted to canonical form before
 * matching.
 * </p>
 * @param string|null $defaultLocale <p>
 * The locale to use if no match is found.
 * </p>
 * @return string|null The closest matching language tag or default value.
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
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Tries to find out best available locale based on HTTP "Accept-Language" header
 * @link https://php.net/manual/en/locale.acceptfromhttp.php
 * @param string $header <p>
 * The string containing the "Accept-Language" header according to format in RFC 2616.
 * </p>
 * @return string|false The corresponding locale identifier.
 * @pure
 */
function locale_accept_from_http(string $header): string|false
{
}

/**
 * Constructs a new message formatter
 * @param string $locale
 * @param string $pattern
 * @return MessageFormatter|null
 * @pure
 */
function msgfmt_create(string $locale, string $pattern): null|MessageFormatter
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Format the message
 * @link https://php.net/manual/en/messageformatter.format.php
 * @param MessageFormatter $formatter
 * @param array $values <p>
 * Arguments to insert into the format string
 * </p>
 * @return string|false The formatted string, or <b>FALSE</b> if an error occurred
 * @pure
 */
function msgfmt_format(MessageFormatter $formatter, array $values): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Quick format message
 * @link https://php.net/manual/en/messageformatter.formatmessage.php
 * @param string $locale <p>
 * The locale to use for formatting locale-dependent parts
 * </p>
 * @param string $pattern <p>
 * The pattern string to insert things into.
 * The pattern uses an 'apostrophe-friendly' syntax; it is run through
 * umsg_autoQuoteApostrophe
 * before being interpreted.
 * </p>
 * @param array $values <p>
 * The array of values to insert into the format string
 * </p>
 * @return string|false The formatted pattern string or <b>FALSE</b> if an error occurred
 * @pure
 */
function msgfmt_format_message(string $locale, string $pattern, array $values): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Parse input string according to pattern
 * @link https://php.net/manual/en/messageformatter.parse.php
 * @param MessageFormatter $formatter
 * @param string $string <p>
 * The string to parse
 * </p>
 * @return array|false An array containing the items extracted, or <b>FALSE</b> on error
 * @pure
 */
function msgfmt_parse(MessageFormatter $formatter, string $string): array|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Quick parse input string
 * @link https://php.net/manual/en/messageformatter.parsemessage.php
 * @param string $locale <p>
 * The locale to use for parsing locale-dependent parts
 * </p>
 * @param string $pattern <p>
 * The pattern with which to parse the <i>value</i>.
 * </p>
 * @param string $message <p>
 * The string to parse, conforming to the <i>pattern</i>.
 * </p>
 * @return array|false An array containing items extracted, or <b>FALSE</b> on error
 * @pure
 */
function msgfmt_parse_message(string $locale, string $pattern, string $message): array|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Set the pattern used by the formatter
 * @link https://php.net/manual/en/messageformatter.setpattern.php
 * @param MessageFormatter $formatter
 * @param string $pattern <p>
 * The pattern string to use in this message formatter.
 * The pattern uses an 'apostrophe-friendly' syntax; it is run through
 * umsg_autoQuoteApostrophe
 * before being interpreted.
 * </p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function msgfmt_set_pattern(MessageFormatter $formatter, string $pattern): bool
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get the pattern used by the formatter
 * @link https://php.net/manual/en/messageformatter.getpattern.php
 * @param MessageFormatter $formatter
 * @return string|false The pattern string for this message formatter
 * @pure
 */
function msgfmt_get_pattern(MessageFormatter $formatter): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get the locale for which the formatter was created.
 * @link https://php.net/manual/en/messageformatter.getlocale.php
 * @param MessageFormatter $formatter
 * @return string The locale name
 * @pure
 */
function msgfmt_get_locale(MessageFormatter $formatter): string
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get the error code from last operation
 * @link https://php.net/manual/en/messageformatter.geterrorcode.php
 * @param MessageFormatter $formatter
 * @return int The error code, one of UErrorCode values. Initial value is U_ZERO_ERROR.
 */
function msgfmt_get_error_code(MessageFormatter $formatter): int
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get the error text from the last operation
 * @link https://php.net/manual/en/messageformatter.geterrormessage.php
 * @param MessageFormatter $formatter
 * @return string Description of the last error.
 */
function msgfmt_get_error_message(MessageFormatter $formatter): string
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Create a date formatter
 * @link https://php.net/manual/en/intldateformatter.create.php
 * @param string|null $locale <p>
 * Locale to use when formatting or parsing.
 * </p>
 * @param int $dateType <p>
 * Date type to use (<b>none</b>,
 * <b>short</b>, <b>medium</b>,
 * <b>long</b>, <b>full</b>).
 * This is one of the
 * IntlDateFormatter constants.
 * </p>
 * @param int $timeType <p>
 * Time type to use (<b>none</b>,
 * <b>short</b>, <b>medium</b>,
 * <b>long</b>, <b>full</b>).
 * This is one of the
 * IntlDateFormatter constants.
 * </p>
 * @param string|null $timezone [optional] <p>
 * Time zone ID, default is system default.
 * </p>
 * @param IntlCalendar|int|null $calendar [optional] <p>
 * Calendar to use for formatting or parsing; default is Gregorian.
 * This is one of the
 * IntlDateFormatter calendar constants.
 * </p>
 * @param string|null $pattern [optional] <p>
 * Optional pattern to use when formatting or parsing.
 * Possible patterns are documented at http://userguide.icu-project.org/formatparse/datetime.
 * </p>
 * @return IntlDateFormatter|null
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
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get the datetype used for the IntlDateFormatter
 * @link https://php.net/manual/en/intldateformatter.getdatetype.php
 * @param IntlDateFormatter $formatter
 * @return int|false The current date type value of the formatter.
 * @pure
 */
function datefmt_get_datetype(IntlDateFormatter $formatter): int|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get the timetype used for the IntlDateFormatter
 * @link https://php.net/manual/en/intldateformatter.gettimetype.php
 * @param IntlDateFormatter $formatter
 * @return int|false The current date type value of the formatter.
 * @pure
 */
function datefmt_get_timetype(IntlDateFormatter $formatter): int|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get the calendar type used for the IntlDateFormatter
 * @link https://php.net/manual/en/intldateformatter.getcalendar.php
 * @param IntlDateFormatter $formatter
 * @return int|false The calendar being used by the formatter.
 * @pure
 */
function datefmt_get_calendar(IntlDateFormatter $formatter): int|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * sets the calendar used to the appropriate calendar, which must be
 * @link https://php.net/manual/en/intldateformatter.setcalendar.php
 * @param IntlDateFormatter $formatter $mf
 * @param IntlCalendar|int|null $calendar <p>
 * The calendar to use.
 * Default is <b>IntlDateFormatter::GREGORIAN</b>.
 * </p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function datefmt_set_calendar(IntlDateFormatter $formatter, IntlCalendar|int|null $calendar): bool
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get the locale used by formatter
 * @link https://php.net/manual/en/intldateformatter.getlocale.php
 * @param IntlDateFormatter $formatter
 * @param int $type [optional]
 * @return string|false the locale of this formatter or 'false' if error
 * @pure
 */
function datefmt_get_locale(IntlDateFormatter $formatter, int $type = ULOC_ACTUAL_LOCALE): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get the timezone-id used for the IntlDateFormatter
 * @link https://php.net/manual/en/intldateformatter.gettimezoneid.php
 * @param IntlDateFormatter $formatter
 * @return string|false ID string for the time zone used by this formatter.
 * @pure
 */
function datefmt_get_timezone_id(IntlDateFormatter $formatter): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 3.0.0)<br/>
 * Get copy of formatter's calendar object
 * @link https://secure.php.net/manual/en/intldateformatter.getcalendarobject.php
 * @param IntlDateFormatter $formatter
 * @return IntlCalendar|false|null A copy of the internal calendar object used by this formatter.
 * @pure
 */
function datefmt_get_calendar_object(IntlDateFormatter $formatter): IntlCalendar|false|null
{
}

/**
 * (PHP 5 &gt;= 5.5.0, PECL intl &gt;= 3.0.0)<br/>
 *  Get formatter's timezone
 * @link https://secure.php.net/manual/en/intldateformatter.gettimezone.php
 * @param IntlDateFormatter $formatter
 * @return IntlTimeZone|false The associated IntlTimeZone object or FALSE on failure.
 * @pure
 */
function datefmt_get_timezone(IntlDateFormatter $formatter): IntlTimeZone|false
{
}

/**
 * (PHP 5 &gt;= 5.5.0, PECL intl &gt;= 3.0.0)<br/>
 * Sets formatter's timezone
 * @link https://php.net/manual/en/intldateformatter.settimezone.php
 * @param IntlDateFormatter $formatter
 * @param IntlTimeZone|DateTimeZone|string|null $timezone <p>
 * The timezone to use for this formatter. This can be specified in the
 * following forms:
 * <ul>
 * <li>
 * <p>
 * <b>NULL</b>, in which case the default timezone will be used, as specified in
 * the ini setting {@link "https://secure.php.net/manual/en/datetime.configuration.php#ini.date.timezone" date.timezone} or
 * through the function  {@link "https://secure.php.net/manual/en/function.date-default-timezone-set.php" date_default_timezone_set()} and as
 * returned by {@link "https://secure.php.net/manual/en/function.date-default-timezone-get.php" date_default_timezone_get()}.
 * </p>
 * </li>
 * <li>
 * <p>
 * An {@link "https://secure.php.net/manual/en/class.intltimezone.php" IntlTimeZone}, which will be used directly.
 * </p>
 * </li>
 * <li>
 * <p>
 * A {@link "https://secure.php.net/manual/en/class.datetimezone.php" DateTimeZone}. Its identifier will be extracted
 * and an ICU timezone object will be created; the timezone will be backed
 * by ICU's database, not PHP's.
 * </p>
 * </li>
 * <li>
 * <p>
 * A {@link "https://secure.php.net/manual/en/language.types.string.php" string}, which should be a valid ICU timezone identifier.
 * See <b>IntlTimeZone::createTimeZoneIDEnumeration()</b>. Raw offsets such as <em>"GMT+08:30"</em> are also accepted.
 * </p>
 * </li>
 * </ul>
 * </p>
 * @return bool|null <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function datefmt_set_timezone(IntlDateFormatter $formatter, $timezone): null|bool
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get the pattern used for the IntlDateFormatter
 * @link https://php.net/manual/en/intldateformatter.getpattern.php
 * @param IntlDateFormatter $formatter
 * @return string|false The pattern string being used to format/parse.
 * @pure
 */
function datefmt_get_pattern(IntlDateFormatter $formatter): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Set the pattern used for the IntlDateFormatter
 * @link https://php.net/manual/en/intldateformatter.setpattern.php
 * @param IntlDateFormatter $formatter
 * @param string $pattern <p>
 * New pattern string to use.
 * Possible patterns are documented at http://userguide.icu-project.org/formatparse/datetime.
 * </p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 * Bad formatstrings are usually the cause of the failure.
 */
function datefmt_set_pattern(IntlDateFormatter $formatter, string $pattern): bool
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get the lenient used for the IntlDateFormatter
 * @link https://php.net/manual/en/intldateformatter.islenient.php
 * @param IntlDateFormatter $formatter
 * @return bool <b>TRUE</b> if parser is lenient, <b>FALSE</b> if parser is strict. By default the parser is lenient.
 * @pure
 */
function datefmt_is_lenient(IntlDateFormatter $formatter): bool
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Set the leniency of the parser
 * @link https://php.net/manual/en/intldateformatter.setlenient.php
 * @param IntlDateFormatter $formatter
 * @param bool $lenient <p>
 * Sets whether the parser is lenient or not, default is <b>TRUE</b> (lenient).
 * </p>
 * @return void
 */
function datefmt_set_lenient(IntlDateFormatter $formatter, bool $lenient): void
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Format the date/time value as a string
 * @link https://php.net/manual/en/intldateformatter.format.php
 * @param IntlDateFormatter $formatter
 * @param object|array|string|int|float $datetime <p>
 * Value to format. This may be a <b>DateTime</b> object,
 * an integer representing a Unix timestamp value (seconds
 * since epoch, UTC) or an array in the format output by
 * <b>localtime</b>.
 * </p>
 * @return string|false The formatted string or, if an error occurred, <b>FALSE</b>.
 * @pure
 */
function datefmt_format(IntlDateFormatter $formatter, $datetime): string|false
{
}

/**
 * (PHP 5 &gt;= 5.5.0, PECL intl &gt;= 3.0.0)<br/>
 * Formats an object
 * @link https://secure.php.net/manual/en/intldateformatter.formatobject.php
 * @param IntlCalendar|DateTimeInterface $datetime <p>
 * An object of type IntlCalendar or DateTime. The timezone information in the object will be used.
 * </p>
 * @param array|int|string|null $format [optional] <p>
 * How to format the date/time. This can either be an {https://secure.php.net/manual/en/language.types.array.php array}  with
 * two elements (first the date style, then the time style, these being one
 * of the constants <b>IntlDateFormatter::NONE</b>,
 * <b>IntlDateFormatter::SHORT</b>,
 * <b>IntlDateFormatter::MEDIUM</b>,
 * <b>IntlDateFormatter::LONG</b>,
 * <b>IntlDateFormatter::FULL</b>), a long with
 * the value of one of these constants (in which case it will be used both
 * for the time and the date) or a {@link https://secure.php.net/manual/en/language.types.string.php} with the format
 * described in {@link http://www.icu-project.org/apiref/icu4c/classSimpleDateFormat.html#details the ICU documentation}
 * documentation. If <b>NULL</b>, the default style will be used.
 * </p>
 * @param string|null $locale [optional] <p>
 * The locale to use, or NULL to use the default one.</p>
 * @return string|false The formatted string or, if an error occurred, <b>FALSE</b>.
 * @pure
 */
function datefmt_format_object($datetime, $format = null, null|string $locale = null): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Parse string to a timestamp value
 * @link https://php.net/manual/en/intldateformatter.parse.php
 * @param IntlDateFormatter $formatter
 * @param string $string <p>
 * string to convert to a time
 * </p>
 * @param int &$offset [optional] <p>
 * Position at which to start the parsing in $value (zero-based).
 * If no error occurs before $value is consumed, $parse_pos will contain -1
 * otherwise it will contain the position at which parsing ended (and the error occurred).
 * This variable will contain the end position if the parse fails.
 * If $parse_pos > strlen($value), the parse fails immediately.
 * </p>
 * @return int|float|false timestamp parsed value
 */
function datefmt_parse(IntlDateFormatter $formatter, string $string, &$offset = null): int|float|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Parse string to a field-based time value
 * @link https://php.net/manual/en/intldateformatter.localtime.php
 * @param IntlDateFormatter $formatter
 * @param string $string <p>
 * string to convert to a time
 * </p>
 * @param int &$offset [optional] <p>
 * Position at which to start the parsing in $value (zero-based).
 * If no error occurs before $value is consumed, $parse_pos will contain -1
 * otherwise it will contain the position at which parsing ended .
 * If $parse_pos > strlen($value), the parse fails immediately.
 * </p>
 * @return array|false Localtime compatible array of integers : contains 24 hour clock value in tm_hour field
 */
function datefmt_localtime(IntlDateFormatter $formatter, string $string, &$offset = null): array|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get the error code from last operation
 * @link https://php.net/manual/en/intldateformatter.geterrorcode.php
 * @param IntlDateFormatter $formatter
 * @return int The error code, one of UErrorCode values. Initial value is U_ZERO_ERROR.
 */
function datefmt_get_error_code(IntlDateFormatter $formatter): int
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get the error text from the last operation.
 * @link https://php.net/manual/en/intldateformatter.geterrormessage.php
 * @param IntlDateFormatter $formatter
 * @return string Description of the last error.
 */
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
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Function to extract a sequence of default grapheme clusters from a text buffer, which must be encoded in UTF-8.
 * @link https://php.net/manual/en/function.grapheme-extract.php
 * @param string $haystack <p>
 * String to search.
 * </p>
 * @param int $size <p>
 * Maximum number items - based on the $extract_type - to return.
 * </p>
 * @param int $type <p>
 * Defines the type of units referred to by the $size parameter:
 * </p>
 * <p>
 * GRAPHEME_EXTR_COUNT (default) - $size is the number of default
 * grapheme clusters to extract.
 * GRAPHEME_EXTR_MAXBYTES - $size is the maximum number of bytes
 * returned.
 * GRAPHEME_EXTR_MAXCHARS - $size is the maximum number of UTF-8
 * characters returned.
 * </p>
 * @param int $offset [optional] <p>
 * Starting position in $haystack in bytes - if given, it must be zero or a
 * positive value that is less than or equal to the length of $haystack in
 * bytes. If $start does not point to the first byte of a UTF-8
 * character, the start position is moved to the next character boundary.
 * </p>
 * @param int &$next [optional] <p>
 * Reference to a value that will be set to the next starting position.
 * When the call returns, this may point to the first byte position past the end of the string.
 * </p>
 * @return string|false A string starting at offset $start and ending on a default grapheme cluster
 * boundary that conforms to the $size and $extract_type specified.
 */
function grapheme_extract(string $haystack, int $size, int $type = 0, int $offset = 0, &$next = null): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PHP 7, PECL intl &gt;= 1.0.2, PHP 7, PECL idn &gt;= 0.1)<br/>
 * Convert domain name to IDNA ASCII form.
 * @link https://php.net/manual/en/function.idn-to-ascii.php
 * @param string $domain <p>
 * Domain to convert. In PHP 5 must be UTF-8 encoded.
 * If e.g. an ISO-8859-1 (aka Western Europe latin1) encoded string is
 * passed it will be converted into an ACE encoded "xn--" string.
 * It will not be the one you expected though!
 * </p>
 * @param int $flags [optional] <p>
 * Conversion options - combination of IDNA_* constants (except IDNA_ERROR_* constants).
 * </p>
 * @param int $variant [optional] <p>
 * Either INTL_IDNA_VARIANT_2003 for IDNA 2003 or INTL_IDNA_VARIANT_UTS46 for UTS #46.
 * </p>
 * @param array &$idna_info [optional] <p>
 * This parameter can be used only if INTL_IDNA_VARIANT_UTS46 was used for variant.
 * In that case, it will be filled with an array with the keys 'result',
 * the possibly illegal result of the transformation, 'isTransitionalDifferent',
 * a boolean indicating whether the usage of the transitional mechanisms of UTS #46
 * either has or would have changed the result and 'errors',
 * which is an int representing a bitset of the error constants IDNA_ERROR_*.
 * </p>
 * @return string|false The ACE encoded version of the domain name or <b>FALSE</b> on failure.
 */
function idn_to_ascii(string $domain, int $flags = 0, int $variant = INTL_IDNA_VARIANT_UTS46, &$idna_info): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PHP 7, PECL intl &gt;= 1.0.2, PHP 7, PECL idn &gt;= 0.1)<br/>
 * Convert domain name from IDNA ASCII to Unicode.
 * @link https://php.net/manual/en/function.idn-to-utf8.php
 * @param string $domain <p>
 * Domain to convert in IDNA ASCII-compatible format.
 * The ASCII encoded domain name. Looks like "xn--..." if the it originally contained non-ASCII characters.
 * </p>
 * @param int $flags [optional] <p>
 * Conversion options - combination of IDNA_* constants (except IDNA_ERROR_* constants).
 * </p>
 * @param int $variant [optional] <p>
 * Either INTL_IDNA_VARIANT_2003 for IDNA 2003 or INTL_IDNA_VARIANT_UTS46 for UTS #46.
 * </p>
 * @param array &$idna_info [optional] <p>
 * This parameter can be used only if INTL_IDNA_VARIANT_UTS46 was used for variant.
 * In that case, it will be filled with an array with the keys 'result',
 * the possibly illegal result of the transformation, 'isTransitionalDifferent',
 * a boolean indicating whether the usage of the transitional mechanisms of UTS #46
 * either has or would have changed the result and 'errors',
 * which is an int representing a bitset of the error constants IDNA_ERROR_*.
 * </p>
 * @return string|false The UTF-8 encoded version of the domain name or <b>FALSE</b> on failure.
 * RFC 3490 4.2 states though "ToUnicode never fails. If any step fails, then the original input
 * sequence is returned immediately in that step."
 */
function idn_to_utf8(string $domain, int $flags = 0, int $variant = INTL_IDNA_VARIANT_UTS46, &$idna_info): string|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Create a new IntlCalendar
 * @link https://secure.php.net/manual/en/intlcalendar.createinstance.php
 * @param IntlTimeZone|DateTimeZone|string|null $timezone [optional] <p> <p>
 * The timezone to use.
 * </p>
 *
 * <ul>
 * <li>
 * <p>
 * <b>NULL</b>, in which case the default timezone will be used, as specified in
 * the ini setting {@link https://secure.php.net/manual/en/datetime.configuration.php#ini.date.timezone date.timezone} or
 * through the function  {@link https://secure.php.net/manual/en/function.date-default-timezone-set.php date_default_timezone_set()} and as
 * returned by {@link https://secure.php.net/manual/en/function.date-default-timezone-get.php date_default_timezone_get()}.
 * </p>
 * </li>
 * <li>
 * <p>
 * An {@link https://secure.php.net/manual/en/class.intltimezone.php IntlTimeZone}, which will be used directly.
 * </p>
 * </li>
 * <li>
 * <p>
 * A {@link https://secure.php.net/manual/en/class.datetimezone.php DateTimeZone}. Its identifier will be extracted
 * and an ICU timezone object will be created; the timezone will be backed
 * by ICU's database, not PHP's.
 * </p>
 * </li>
 * <li>
 * <p>
 * A {@link https://secure.php.net/manual/en/language.types.string.php string}, which should be a valid ICU timezone identifier.
 * See  <b>IntlTimeZone::createTimeZoneIDEnumeration()</b>. Raw
 * offsets such as <em>"GMT+08:30"</em> are also accepted.
 * </p>
 * </li>
 * </ul>
 * </p>
 * @param string|null $locale [optional] <p>
 * A locale to use or <b>NULL</b> to use {@link https://secure.php.net/manual/en/intl.configuration.php#ini.intl.default-locale the default locale}.
 * </p>
 * @return IntlCalendar|null
 * The created {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} instance or <b>NULL</b> on
 * failure.
 * @pure
 */
function intlcal_create_instance($timezone = null, null|string $locale = null): null|IntlCalendar
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get set of locale keyword values
 * @param string $keyword <p>
 * The locale keyword for which relevant values are to be queried. Only
 * <em>'calendar'</em> is supported.
 * </p>
 * @param string $locale <p>
 * The locale onto which the keyword/value pair are to be appended.
 * </p>
 * @param bool $onlyCommon
 * <p>
 * Whether to show only the values commonly used for the specified locale.
 * </p>
 * @return IntlIterator|false An iterator that yields strings with the locale keyword values or <b>FALSE</b> on failure.
 * @pure
 */
function intlcal_get_keyword_values_for_locale(string $keyword, string $locale, bool $onlyCommon): IntlIterator|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get number representing the current time
 * @link https://secure.php.net/manual/en/intlcalendar.getnow.php
 * @return float A float representing a number of milliseconds since the epoch, not counting leap seconds.
 * @since 5.5
 */
function intlcal_get_now(): float
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get array of locales for which there is data
 * @link https://secure.php.net/manual/en/intlcalendar.getavailablelocales.php
 * @return string[] An array of strings, one for which locale.
 * @pure
 */
function intlcal_get_available_locales(): array
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get the value for a field
 * @link https://secure.php.net/manual/en/intlcalendar.get.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $field <p>
 * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
 * values between <em>0</em> and
 * <b>IntlCalendar::FIELD_COUNT</b>.
 * </p>
 * @return int An integer with the value of the time field.
 * @pure
 */
function intl_get($calendar, $field)
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get time currently represented by the object
 * @param IntlCalendar $calendar <p>The calendar whose time will be checked against this object's time.</p>
 * @return float
 * A {@link https://secure.php.net/manual/en/language.types.float.php float} representing the number of milliseconds elapsed since the
 * reference time (1 Jan 1970 00:00:00 UTC).
 * @pure
 */
function intlcal_get_time(IntlCalendar $calendar): float|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Set the calendar time in milliseconds since the epoch
 * @link https://secure.php.net/manual/en/intlcalendar.settime.php
 * @param IntlCalendar $calendar <p>
 * The IntlCalendar resource.
 * </p>
 * @param float $timestamp <p>
 * An instant represented by the number of number of milliseconds between
 * such instant and the epoch, ignoring leap seconds.
 * </p>
 * @return bool
 * Returns <b>TRUE</b> on success and <b>FALSE</b> on failure.
 * @since 5.5
 */
function intlcal_set_time(IntlCalendar $calendar, float $timestamp): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Add a (signed) amount of time to a field
 * @link https://secure.php.net/manual/en/intlcalendar.add.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $field <p>
 * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}.
 * These are integer values between <em>0</em> and
 * <b>IntlCalendar::FIELD_COUNT</b>.
 * </p>
 * @param int $value <p>The signed amount to add to the current field. If the amount is positive, the instant will be moved forward; if it is negative, the instant wil be moved into the past. The unit is implicit to the field type.
 * For instance, hours for IntlCalendar::FIELD_HOUR_OF_DAY.</p>
 * @return bool Returns <b>TRUE</b> on success or <b>FALSE</b> on failure.
 * @since 5.5
 */
function intlcal_add(IntlCalendar $calendar, int $field, int $value): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Set the timezone used by this calendar
 * @link https://secure.php.net/manual/en/intlcalendar.settimezone.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param IntlTimeZone|DateTimeZone|string|null $timezone <p>
 * The new timezone to be used by this calendar. It can be specified in the
 * following ways:
 *
 * </p><ul>
 * <li>
 * <p>
 * <b>NULL</b>, in which case the default timezone will be used, as specified in
 * the ini setting {@link https://secure.php.net/manual/en/datetime.configuration.php#ini.date.timezone date.timezone} or
 * through the function  {@link https://secure.php.net/manual/en/function.date-default-timezone-set.php date_default_timezone_set()} and as
 * returned by  {@link https://secure.php.net/manual/en/function.date-default-timezone-get.php date_default_timezone_get()}.
 * </p>
 * </li>
 * <li>
 * <p>
 * An {@link https://secure.php.net/manual/en/class.intltimezone.php IntlTimeZone}, which will be used directly.
 * </p>
 * </li>
 * <li>
 * <p>
 * A {@link https://secure.php.net/manual/en/class.datetimezone.php DateTimeZone}. Its identifier will be extracted
 * and an ICU timezone object will be created; the timezone will be backed
 * by ICU's database, not PHP's.
 * </p>
 * </li>
 * <li>
 * <p>
 * A {@link https://secure.php.net/manual/en/language.types.string.php string}, which should be a valid ICU timezone identifier.
 * See  <b>IntlTimeZone::createTimeZoneIDEnumeration()</b>. Raw
 * offsets such as <em>"GMT+08:30"</em> are also accepted.
 * </p>
 * </li>
 * </ul>
 * @return bool Returns <b>TRUE</b> on success and <b>FALSE</b> on failure.
 * @since 5.5
 */
function intlcal_set_time_zone(IntlCalendar $calendar, $timezone): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Whether this object's time is after that of the passed object
 * https://secure.php.net/manual/en/intlcalendar.after.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param IntlCalendar $other <p>The calendar whose time will be checked against this object's time.</p>
 * @return bool
 * Returns <b>TRUE</b> if this object's current time is after that of the
 * <em>calendar</em> argument's time. Returns <b>FALSE</b> otherwise.
 * Also returns <b>FALSE</b> on failure. You can use {@link https://secure.php.net/manual/en/intl.configuration.php#ini.intl.use-exceptions exceptions} or
 * {@link https://secure.php.net/manual/en/function.intl-get-error-code.php intl_get_error_code()} to detect error conditions.
 * @pure
 */
function intlcal_after(IntlCalendar $calendar, IntlCalendar $other): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Whether this object's time is before that of the passed object
 * @link https://secure.php.net/manual/en/intlcalendar.before.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param IntlCalendar $other <p> The calendar whose time will be checked against this object's time.</p>
 * @return bool
 * <p>
 * Returns <b>TRUE</B> if this object's current time is before that of the
 * <em>calendar</em> argument's time. Returns <b>FALSE</b> otherwise.
 * Also returns <b>FALSE</b> on failure. You can use {@link https://secure.php.net/manual/en/intl.configuration.php#ini.intl.use-exceptions exceptions} or
 * {@link https://secure.php.net/manual/en/function.intl-get-error-code.php intl_get_error_code()} to detect error conditions.
 * </p>
 * @pure
 */
function intlcal_before(IntlCalendar $calendar, IntlCalendar $other): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Set a time field or several common fields at once
 * @link https://secure.php.net/manual/en/intlcalendar.set.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $year <p>
 * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
 * values between <em>0</em> and
 * <b>IntlCalendar::FIELD_COUNT</b>.
 * </p>
 * @param int $month <p>
 * The new value for <b>IntlCalendar::FIELD_MONTH</b>.
 * </p>
 * @param int $dayOfMonth [optional] <p>
 * The new value for <b>IntlCalendar::FIELD_DAY_OF_MONTH</b>.
 * The month sequence is zero-based, i.e., January is represented by 0,
 * February by 1, ..., December is 11 and Undecember (if the calendar has
 * it) is 12.
 * </p>
 * @param int $hour [optional]
 * <p>
 * The new value for <b>IntlCalendar::FIELD_HOUR_OF_DAY</b>.
 * </p>
 * @param int $minute [optional]
 * <p>
 * The new value for <b>IntlCalendar::FIELD_MINUTE</b>.
 * </p>
 * @param int $second [optional] <p>
 * The new value for <b>IntlCalendar::FIELD_SECOND</b>.
 * </p>
 * @return bool Returns <b>TRUE</b> on success and <b>FALSE</b> on failure.
 * @since 5.5
 */
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

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Add value to field without carrying into more significant fields
 * @link https://secure.php.net/manual/en/intlcalendar.roll.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $field <p>One of the
 * {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time
 * {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}.
 * These are integer values between <em>0</em> and
 * <b>IntlCalendar::FIELD_COUNT</b>.
 * </p>
 * @param int|bool $value <p>
 * The (signed) amount to add to the field, <b>TRUE</b> for rolling up (adding
 * <em>1</em>), or <b>FALSE</b> for rolling down (subtracting
 * <em>1</em>).
 * </p>
 * @return bool Returns <b>TRUE</b> on success or <b>FALSE</b> on failure.
 * @since 5.5
 */
function intlcal_roll(IntlCalendar $calendar, int $field, int|bool $value): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Clear a field or all fields
 * @link https://secure.php.net/manual/en/intlcalendar.clear.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int|null $field [optional] <p>
 * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
 * values between <em>0</em> and
 * <b>IntlCalendar::FIELD_COUNT</b>.
 * </p>
 * @return bool Returns <b>TRUE</b> on success or <b>FALSE</b> on failure. Failure can only occur is invalid arguments are provided.
 * @since 5.5
 */
#[LanguageAware(['8.3' => 'true'], default: 'bool')]
function intlcal_clear(IntlCalendar $calendar, null|int $field = null): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Calculate difference between given time and this object's time
 * @link https://secure.php.net/manual/en/intlcalendar.fielddifference.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param float $timestamp <p>
 * The time against which to compare the quantity represented by the
 * <em>field</em>. For the result to be positive, the time
 * given for this parameter must be ahead of the time of the object the
 * method is being invoked on.
 * </p>
 * @param int $field <p>
 * The field that represents the quantity being compared.
 * </p>
 *
 * <p>
 * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
 * values between <em>0</em> and
 * <b>IntlCalendar::FIELD_COUNT</b>.
 * </p>
 * @return int Returns a (signed) difference of time in the unit associated with the
 * specified field or <b>FALSE</b> on failure.
 * @pure
 */
#[LanguageAware(['8.0' => 'int|false'], default: 'int')]
function intlcal_field_difference(IntlCalendar $calendar, float $timestamp, int $field)
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * The maximum value for a field, considering the object's current time
 * @link https://secure.php.net/manual/en/intlcalendar.getactualmaximum.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $field <p>
 * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
 * values between <em>0</em> and
 * <b>IntlCalendar::FIELD_COUNT</b>.
 * </p>
 * @return int
 * An {@link https://secure.php.net/manual/en/language.types.integer.php int} representing the maximum value in the units associated
 * with the given <em>field</em> or <b>FALSE</b> on failure.
 * @pure
 */
#[LanguageAware(['8.0' => 'int|false'], default: 'int')]
function intlcal_get_actual_maximum(IntlCalendar $calendar, int $field)
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * The minimum value for a field, considering the object's current time
 * @link https://secure.php.net/manual/en/intlcalendar.getactualminimum.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $field <p>
 * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}.
 * These are integer values between <em>0</em> and
 * <b>IntlCalendar::FIELD_COUNT</b>.
 * </p>
 * @return int
 * An {@link https://secure.php.net/manual/en/language.types.integer.php int} representing the minimum value in the field's
 * unit or <b>FALSE</b> on failure.
 * @pure
 */
#[LanguageAware(['8.0' => 'int|false'], default: 'int')]
function intlcal_get_actual_minimum(IntlCalendar $calendar, int $field)
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * @link https://secure.php.net/manual/en/intlcalendar.getdayofweektype.php
 * Tell whether a day is a weekday, weekend or a day that has a transition between the two
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $dayOfWeek <p>
 * One of the constants <b>IntlCalendar::DOW_SUNDAY</b>,
 * <b>IntlCalendar::DOW_MONDAY</b>, ...,
 * <b>IntlCalendar::DOW_SATURDAY</b>.
 * </p>
 * @return int
 * Returns one of the constants
 * <b>IntlCalendar::DOW_TYPE_WEEKDAY</b>,
 * <b>IntlCalendar::DOW_TYPE_WEEKEND</b>,
 * <b>IntlCalendar::DOW_TYPE_WEEKEND_OFFSET</b> or
 * <b>IntlCalendar::DOW_TYPE_WEEKEND_CEASE</b> or <b>FALSE</b> on failure.
 * @pure
 */
#[LanguageAware(['8.0' => 'int|false'], default: 'int')]
function intlcal_get_day_of_week_type(IntlCalendar $calendar, int $dayOfWeek)
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get the first day of the week for the calendar's locale
 * @link https://secure.php.net/manual/en/intlcalendar.getfirstdayofweek.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @return int
 * One of the constants <b>IntlCalendar::DOW_SUNDAY</b>,
 * <b>IntlCalendar::DOW_MONDAY</b>, ...,
 * <b>IntlCalendar::DOW_SATURDAY</b> or <b>FALSE</b> on failure.
 * @pure
 */
#[LanguageAware(['8.0' => 'int|false'], default: 'int')]
function intlcal_get_first_day_of_week(IntlCalendar $calendar)
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get the largest local minimum value for a field
 * @link https://secure.php.net/manual/en/intlcalendar.getgreatestminimum.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $field <p>
 * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
 * values between <em>0</em> and
 * <b>IntlCalendar::FIELD_COUNT</b>.</p>
 * @return int
 * An {@link https://secure.php.net/manual/en/language.types.integer.php int} representing a field value, in the field's
 * unit, or <b>FALSE</b> on failure.
 * @pure
 */
function intlcal_greates_minimum($calendar, $field)
{
}

/**
 * (PHP &gt;= 5.5.0, PECL intl &gt;= 3.0.0a1)<br/>
 * Gets the value for a specific field.
 * @link https://www.php.net/manual/en/intlcalendar.get.php
 * @param IntlCalendar $calendar <p>
 * The IntlCalendar resource.
 * </p>
 * @param int $field <p>
 * One of the IntlCalendar date/time field constants. These are integer values between 0 and IntlCalendar::FIELD_COUNT.
 * </p>
 * @return int An integer with the value of the time field.
 * @pure
 */
#[LanguageAware(['8.0' => 'int|false'], default: 'int')]
function intlcal_get(IntlCalendar $calendar, int $field)
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get the smallest local maximum for a field
 * @link https://secure.php.net/manual/en/intlcalendar.getleastmaximum.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $field <p>
 * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
 * values between <em>0</em> and
 * <b>IntlCalendar::FIELD_COUNT</b>.
 * </p>
 * @return int
 * <p>An {@link https://secure.php.net/manual/en/language.types.integer.ph int} representing a field value in the field's
 * unit or <b>FALSE</b> on failure.
 * </p>
 * @pure
 */
#[LanguageAware(['8.0' => 'int|false'], default: 'int')]
function intlcal_get_least_maximum(IntlCalendar $calendar, int $field)
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get the largest local minimum value for a field
 * @link https://secure.php.net/manual/en/intlcalendar.getgreatestminimum.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $field <p>
 * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
 * values between <em>0</em> and
 * <b>IntlCalendar::FIELD_COUNT</b>.</p>
 * @return int
 * An {@link https://secure.php.net/manual/en/language.types.integer.php int} representing a field value, in the field's
 * unit, or <b>FALSE</b> on failure.
 * @pure
 */
#[LanguageAware(['8.0' => 'int|false'], default: 'int')]
function intlcal_get_greatest_minimum(IntlCalendar $calendar, int $field)
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get the locale associated with the object
 * @link https://secure.php.net/manual/en/intlcalendar.getlocale.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $type <p>
 * Whether to fetch the actual locale (the locale from which the calendar
 * data originates, with <b>Locale::ACTUAL_LOCALE</b>) or the
 * valid locale, i.e., the most specific locale supported by ICU relatively
 * to the requested locale – see <b>Locale::VALID_LOCALE</b>.
 * From the most general to the most specific, the locales are ordered in
 * this fashion – actual locale, valid locale, requested locale.
 * </p>
 * @return string
 * A locale string or <b>FALSE</b> on failure.
 * @pure
 */
#[LanguageAware(['8.0' => 'string|false'], default: 'string')]
function intlcal_get_locale(IntlCalendar $calendar, int $type)
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get the global maximum value for a field
 * @link https://secure.php.net/manual/en/intlcalendar.getmaximum.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $field <p>
 * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
 * values between <em>0</em> and
 * <b>IntlCalendar::FIELD_COUNT</b>.
 * </p>
 * @return int|false
 * @pure
 */
function intcal_get_maximum($calendar, $field)
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * @link https://secure.php.net/manual/en/intlcalendar.getminimaldaysinfirstweek.php
 * Get minimal number of days the first week in a year or month can have
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @return int
 * An {@link https://secure.php.net/manual/en/language.types.integer.php  int} representing a number of days or <b>FALSE</b> on failure.
 * @pure
 */
#[LanguageAware(['8.0' => 'int|false'], default: 'int')]
function intlcal_get_minimal_days_in_first_week(IntlCalendar $calendar)
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get the global minimum value for a field
 * @link https://secure.php.net/manual/en/intlcalendar.getminimum.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $field <p>
 * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field}. These are integer
 * values between <em>0</em> and
 * <b>IntlCalendar::FIELD_COUNT</b>.
 * </p>
 * @return int
 * An int representing a value for the given field in the field's unit or FALSE on failure.
 * @pure
 */
#[LanguageAware(['8.0' => 'int|false'], default: 'int')]
function intlcal_get_minimum(IntlCalendar $calendar, int $field)
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get the object's timezone
 * @link https://secure.php.net/manual/en/intlcalendar.gettimezone.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @return IntlTimeZone|false
 * An {@link https://secure.php.net/manual/en/class.intltimezone.php IntlTimeZone} object corresponding to the one used
 * internally in this object.
 * @pure
 */
function intlcal_get_time_zone(IntlCalendar $calendar): IntlTimeZone|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get the calendar type
 * @link https://secure.php.net/manual/en/intlcalendar.gettype.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @return string
 * A {@link https://secure.php.net/manual/en/language.types.string.php string} representing the calendar type, such as
 * <em>'gregorian'</em>, <em>'islamic'</em>, etc.
 * @pure
 */
function intlcal_get_type(IntlCalendar $calendar): string
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get time of the day at which weekend begins or ends
 * @link https://secure.php.net/manual/en/intlcalendar.getweekendtransition.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $dayOfWeek <p>
 * One of the constants <b>IntlCalendar::DOW_SUNDAY</b>,
 * <b>IntlCalendar::DOW_MONDAY</b>, ...,
 * <b>IntlCalendar::DOW_SATURDAY</b>.
 * </p>
 * @return int
 * The number of milliseconds into the day at which the the weekend begins or
 * ends or <b>FALSE</b> on failure.
 * @pure
 */
function intlcal_get_weekend_transition(IntlCalendar $calendar, int $dayOfWeek): int|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Whether the object's time is in Daylight Savings Time
 * @link https://secure.php.net/manual/en/intlcalendar.indaylighttime.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @return bool
 * Returns <b>TRUE</b> if the date is in Daylight Savings Time, <b>FALSE</b> otherwise.
 * The value <b>FALSE</b> may also be returned on failure, for instance after
 * specifying invalid field values on non-lenient mode; use {@link https://secure.php.net/manual/en/intl.configuration.php#ini.intl.use-exceptions exceptions} or query
 * {@link https://secure.php.net/manual/en/function.intl-get-error-code.php intl_get_error_code()} to disambiguate.
 * @pure
 */
function intlcal_in_daylight_time(IntlCalendar $calendar): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Whether date/time interpretation is in lenient mode
 * @link https://secure.php.net/manual/en/intlcalendar.islenient.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @return bool
 * A {@link https://secure.php.net/manual/en/language.types.boolean.php bool} representing whether the calendar is set to lenient mode.
 * @pure
 */
function intlcal_is_lenient(IntlCalendar $calendar): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Whether a field is set
 * @link https://secure.php.net/manual/en/intlcalendar.isset.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $field <p>
 * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
 * values between <em>0</em> and
 * <b>IntlCalendar::FIELD_COUNT</b>.
 * </p>
 * @return bool Assuming there are no argument errors, returns <b>TRUE</b> iif the field is set.
 * @pure
 */
function intlcal_is_set(IntlCalendar $calendar, int $field): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get the global maximum value for a field
 * @link https://secure.php.net/manual/en/intlcalendar.getmaximum.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $field <p>
 * One of the {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} date/time {@link https://secure.php.net/manual/en/class.intlcalendar.php#intlcalendar.constants field constants}. These are integer
 * values between <em>0</em> and
 * <b>IntlCalendar::FIELD_COUNT</b>.
 * </p>
 * @return int|false
 * @pure
 */
function intlcal_get_maximum(IntlCalendar $calendar, int $field): false|int
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Whether another calendar is equal but for a different time
 * @link https://secure.php.net/manual/en/intlcalendar.isequivalentto.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param IntlCalendar $other The other calendar against which the comparison is to be made.
 * @return bool
 * Assuming there are no argument errors, returns <b>TRUE</b> iif the calendars are equivalent except possibly for their set time.
 * @pure
 */
function intlcal_is_equivalent_to(IntlCalendar $calendar, IntlCalendar $other): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Whether a certain date/time is in the weekend
 * @link https://secure.php.net/manual/en/intlcalendar.isweekend.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param float|null $timestamp [optional] <p>
 * An optional timestamp representing the number of milliseconds since the
 * epoch, excluding leap seconds. If <b>NULL</b>, this object's current time is
 * used instead.
 * </p>
 * @return bool
 * <p> A {@link https://secure.php.net/manual/en/language.types.boolean.php bool} indicating whether the given or this object's time occurs
 * in a weekend.
 * </p>
 * <p>
 * The value <b>FALSE</b> may also be returned on failure, for instance after giving
 * a date out of bounds on non-lenient mode; use {@link https://secure.php.net/manual/en/intl.configuration.php#ini.intl.use-exceptions exceptions} or query
 * {@link https://secure.php.net/manual/en/function.intl-get-error-code.php intl_get_error_code()} to disambiguate.</p>
 * @pure
 */
function intlcal_is_weekend(IntlCalendar $calendar, null|float $timestamp = null): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Set the day on which the week is deemed to start
 * @link https://secure.php.net/manual/en/intlcalendar.setfirstdayofweek.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $dayOfWeek <p>
 * One of the constants <b>IntlCalendar::DOW_SUNDAY</b>,
 * <b>IntlCalendar::DOW_MONDAY</b>, ...,
 * <b>IntlCalendar::DOW_SATURDAY</b>.
 * </p>
 * @return bool Returns TRUE on success. Failure can only happen due to invalid parameters.
 * @since 5.5
 */
function intlcal_set_first_day_of_week(IntlCalendar $calendar, int $dayOfWeek): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Set whether date/time interpretation is to be lenient
 * @link https://secure.php.net/manual/en/intlcalendar.setlenient.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param bool $lenient <p>
 * Use <b>TRUE</b> to activate the lenient mode; <b>FALSE</b> otherwise.
 * </p>
 * @return bool Returns <b>TRUE</b> on success. Failure can only happen due to invalid parameters.
 * @since 5.5
 */
function intlcal_set_lenient(IntlCalendar $calendar, bool $lenient): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get behavior for handling repeating wall time
 * @link https://secure.php.net/manual/en/intlcalendar.getrepeatedwalltimeoption.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @return int
 * One of the constants <b>IntlCalendar::WALLTIME_FIRST</b> or
 * <b>IntlCalendar::WALLTIME_LAST</b>.
 * @pure
 */
function intlcal_get_repeated_wall_time_option(IntlCalendar $calendar): int
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Compare time of two IntlCalendar objects for equality
 * @link https://secure.php.net/manual/en/intlcalendar.equals.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param IntlCalendar $other
 * @return bool <p>
 * Returns <b>TRUE</b> if the current time of both this and the passed in
 * {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} object are the same, or <b>FALSE</b>
 * otherwise. The value <b>FALSE</b> can also be returned on failure. This can only
 * happen if bad arguments are passed in. In any case, the two cases can be
 * distinguished by calling  {@link https://secure.php.net/manual/en/function.intl-get-error-code.php intl_get_error_code()}.
 * </p>
 * @pure
 */
function intlcal_equals(IntlCalendar $calendar, IntlCalendar $other): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get behavior for handling skipped wall time
 * @link https://secure.php.net/manual/en/intlcalendar.getskippedwalltimeoption.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @return int
 * One of the constants <b>IntlCalendar::WALLTIME_FIRST</b>,
 * <b>IntlCalendar::WALLTIME_LAST</b> or
 * <b>IntlCalendar::WALLTIME_NEXT_VALID</b>.
 * @pure
 */
function intlcal_get_skipped_wall_time_option(IntlCalendar $calendar): int
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Set behavior for handling repeating wall times at negative timezone offset transitions
 * @link https://secure.php.net/manual/en/intlcalendar.setrepeatedwalltimeoption.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $option <p>
 * One of the constants <b>IntlCalendar::WALLTIME_FIRST</b> or
 * <b>IntlCalendar::WALLTIME_LAST</b>.
 * </p>
 * @return bool
 * Returns <b>TRUE</b> on success. Failure can only happen due to invalid parameters.
 * @since 5.5
 */
function intlcal_set_repeated_wall_time_option(IntlCalendar $calendar, int $option): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Set behavior for handling skipped wall times at positive timezone offset transitions
 * @link https://secure.php.net/manual/en/intlcalendar.setskippedwalltimeoption.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @param int $option <p>
 * One of the constants <b>IntlCalendar::WALLTIME_FIRST</b>,
 * <b>IntlCalendar::WALLTIME_LAST</b> or
 * <b>IntlCalendar::WALLTIME_NEXT_VALID</b>.
 * </p>
 * @return bool
 * <p>
 * Returns <b>TRUE</b> on success. Failure can only happen due to invalid parameters.
 * </p>
 * @since 5.5
 */
function intlcal_set_skipped_wall_time_option(IntlCalendar $calendar, int $option): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a2)<br/>
 * Create an IntlCalendar from a DateTime object or string
 * @link https://secure.php.net/manual/en/intlcalendar.fromdatetime.php
 * @param DateTime|string $datetime <p>
 * A {@link https://secure.php.net/manual/en/class.datetime.php DateTime} object or a {@link https://secure.php.net/manual/en/language.types.string.php string} that
 * can be passed to  {@link https://secure.php.net/manual/en/datetime.construct.php DateTime::__construct()}.
 * </p>
 * @param null|string $locale
 * @return IntlCalendar|null
 * The created {@link https://secure.php.net/manual/en/class.intlcalendar.php IntlCalendar} object or <b>NULL</b> in case of
 * failure. If a {@link https://secure.php.net/manual/en/language.types.string.php string} is passed, any exception that occurs
 * inside the {@link https://secure.php.net/manual/en/class.datetime.php DateTime} constructor is propagated.
 * @pure
 */
function intlcal_from_date_time(DateTime|string $datetime, null|string $locale = null): null|IntlCalendar
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a2)<br/>
 * Convert an IntlCalendar into a DateTime object
 * @link https://secure.php.net/manual/en/intlcalendar.todatetime.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @return DateTime|false
 * A {@link https://secure.php.net/manual/en/class.datetime.php DateTime} object with the same timezone as this
 * object (though using PHP's database instead of ICU's) and the same time,
 * except for the smaller precision (second precision instead of millisecond).
 * Returns <b>FALSE</b> on failure.
 * @pure
 */
function intlcal_to_date_time(IntlCalendar $calendar): DateTime|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get last error code on the object
 * @link https://secure.php.net/manual/en/intlcalendar.geterrorcode.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @return int|false An ICU error code indicating either success, failure or a warning.
 * @since 5.5
 */
function intlcal_get_error_code(IntlCalendar $calendar): int|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get last error message on the object
 * @link https://secure.php.net/manual/en/intlcalendar.geterrormessage.php
 * @param IntlCalendar $calendar <p>
 * The calendar object, on the procedural style interface.
 * </p>
 * @return string|false The error message associated with last error that occurred in a function call on this object, or a string indicating the non-existance of an error.
 * @since 5.5
 */
function intlcal_get_error_message(IntlCalendar $calendar): string|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get the number of IDs in the equivalency group that includes the given ID
 * @link https://secure.php.net/manual/en/intltimezone.countequivalentids.php
 * @param string $timezoneId
 * @return int|false
 * @pure
 */
function intltz_count_equivalent_ids(string $timezoneId): int|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Create a new copy of the default timezone for this host
 * @link https://secure.php.net/manual/en/intltimezone.createdefault.php
 * @return IntlTimeZone
 * @pure
 */
function intlz_create_default()
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * @link https://secure.php.net/manual/en/intltimezone.createenumeration.php
 * @param IntlTimeZone|string|int|float|null $countryOrRawOffset [optional]
 * @return IntlIterator|false
 * @pure
 */
function intltz_create_enumeration($countryOrRawOffset): IntlIterator|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * @link https://secure.php.net/manual/en/intltimezone.createtimezone.php
 * @param string $timezoneId
 * @return IntlTimeZone|null
 * @pure
 */
function intltz_create_time_zone(string $timezoneId): null|IntlTimeZone
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * @link https://secure.php.net/manual/en/intltimezone.fromdatetimezone.php
 * @param DateTimeZone $timezone
 * @return IntlTimeZone|null
 * @pure
 */
function intltz_from_date_time_zone(DateTimeZone $timezone): null|IntlTimeZone
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get the canonical system timezone ID or the normalized custom time zone ID for the given time zone ID
 * @link https://secure.php.net/manual/en/intltimezone.getcanonicalid.php
 * @param string $timezoneId
 * @param bool &$isSystemId [optional]
 * @return string|false
 * @pure
 */
function intltz_get_canonical_id(string $timezoneId, &$isSystemId): string|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get a name of this time zone suitable for presentation to the user
 * @param IntlTimeZone $timezone - <p>
 * The time zone object, on the procedural style interface.
 * </p>
 * @param bool $dst [optional]
 * @param int $style [optional]
 * @param string|null $locale [optional]
 * @return string|false
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
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get the amount of time to be added to local standard time to get local wall clock time
 * @param IntlTimeZone $timezone - <p>
 * The time zone object, on the procedural style interface.
 * </p>
 * @return int
 * @link https://secure.php.net/manual/en/intltimezone.getequivalentid.php
 * @pure
 */
function intltz_get_dst_savings(IntlTimeZone $timezone): int
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get an ID in the equivalency group that includes the given ID
 * @link https://secure.php.net/manual/en/intltimezone.getequivalentid.php
 * @param string $timezoneId
 * @param int $offset
 * @return string|false
 * @pure
 */
function intltz_get_equivalent_id(string $timezoneId, int $offset): string|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get last error code on the object
 * @link https://secure.php.net/manual/en/intltimezone.geterrorcode.php
 * @param IntlTimeZone $timezone - <p>
 * The time zone object, on the procedural style interface.
 * </p>
 * @return int|false
 * @since 5.5
 */
function intltz_get_error_code(IntlTimeZone $timezone): int|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get last error message on the object
 * @link https://secure.php.net/manual/en/intltimezone.geterrormessage.php
 * @param IntlTimeZone $timezone - <p>
 * The time zone object, on the procedural style interface.
 * </p>
 * @return string|false
 * @since 5.5
 */
function intltz_get_error_message(IntlTimeZone $timezone): string|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Create GMT (UTC) timezone
 * @link https://secure.php.net/manual/en/intltimezone.getgmt.php
 * @return IntlTimeZone
 * @pure
 */
function intltz_getGMT(): IntlTimeZone
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get timezone ID
 * @link https://secure.php.net/manual/en/intltimezone.getid.php
 * @param IntlTimeZone $timezone
 * @return string|false
 * @pure
 */
function intltz_get_id(IntlTimeZone $timezone): string|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get the time zone raw and GMT offset for the given moment in time
 * @link https://secure.php.net/manual/en/intltimezone.getoffset.php
 * @param IntlTimeZone $timezone
 * @param float $timestamp
 * @param bool $local
 * @param int &$rawOffset
 * @param int &$dstOffset
 * @return bool
 * @pure
 */
function intltz_get_offset(IntlTimeZone $timezone, float $timestamp, bool $local, &$rawOffset, &$dstOffset): bool
{
}

/**
 * Get the raw GMT offset (before taking daylight savings time into account
 * @link https://secure.php.net/manual/en/intltimezone.getrawoffset.php
 * @param IntlTimeZone $timezone
 * @return int
 * @pure
 */
function intltz_get_raw_offset(IntlTimeZone $timezone): int
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Get the timezone data version currently used by ICU
 * @link https://secure.php.net/manual/en/intltimezone.gettzdataversion.php
 * @return string|false
 * @pure
 */
function intltz_get_tz_data_version(): string|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Check if this zone has the same rules and offset as another zone
 * @link https://secure.php.net/manual/en/intltimezone.hassamerules.php
 * @param IntlTimeZone $timezone
 * @param IntlTimeZone $other
 * @return bool
 * @pure
 */
function intltz_has_same_rules(IntlTimeZone $timezone, IntlTimeZone $other): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Convert to DateTimeZone object
 * @link https://secure.php.net/manual/en/intltimezone.todatetimezone.php
 * @param IntlTimeZone $timezone
 * @return DateTimeZone|false
 * @pure
 */
function intltz_to_date_time_zone(IntlTimeZone $timezone): DateTimeZone|false
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * Check if this time zone uses daylight savings time
 * @link https://secure.php.net/manual/en/intltimezone.usedaylighttime.php
 * @param IntlTimeZone $timezone
 * @return bool
 * @pure
 */
function intltz_use_daylight_time(IntlTimeZone $timezone): bool
{
}

/**
 * (PHP 5 &gt;=5.5.0 PECL intl &gt;= 3.0.0a1)<br/>
 * @param DateTimeZone|IntlTimeZone|string|int|null $timezoneOrYear [optional]
 * @param string|null $localeOrMonth [optional]
 * @param int $day [optional]
 * @param int $hour [optional]
 * @param int $minute [optional]
 * @param int $second [optional]
 * @return IntlGregorianCalendar|null
 * @pure
 */
#[Deprecated(
    reason: 'use IntlGregorianCalendar::__construct(), IntlGregorianCalendar::createFromDate(), or IntlGregorianCalendar::createFromDateTime() instead',
    since: '8.4',
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

/**
 * @param IntlGregorianCalendar $calendar
 * @param float $timestamp
 * @return bool
 */
function intlgregcal_set_gregorian_change(IntlGregorianCalendar $calendar, float $timestamp): bool
{
}

/**
 * @param IntlGregorianCalendar $calendar
 * @return float
 * @pure
 */
function intlgregcal_get_gregorian_change(IntlGregorianCalendar $calendar): float
{
}

/**
 * @param IntlGregorianCalendar $calendar
 * @param int $year
 * @return bool
 * @pure
 */
function intlgregcal_is_leap_year(IntlGregorianCalendar $calendar, int $year): bool
{
}

/**
 * (PHP &gt;= 5.3.2, PECL intl &gt;= 2.0.0)<br/>
 * Create a resource bundle
 * @link https://php.net/manual/en/resourcebundle.create.php
 * @param string|null $locale <p>
 * Locale for which the resources should be loaded (locale name, e.g. en_CA).
 * </p>
 * @param string|null $bundle <p>
 * The directory where the data is stored or the name of the .dat file.
 * </p>
 * @param bool $fallback [optional] <p>
 * Whether locale should match exactly or fallback to parent locale is allowed.
 * </p>
 * @return ResourceBundle|null <b>ResourceBundle</b> object or <b>NULL</b> on error.
 * @pure
 */
function resourcebundle_create(null|string $locale, null|string $bundle, bool $fallback = true): null|ResourceBundle
{
}

/**
 * (PHP &gt;= 5.3.2, PECL intl &gt;= 2.0.0)<br/>
 * Get data from the bundle
 * @link https://php.net/manual/en/resourcebundle.get.php
 * @param ResourceBundle $bundle
 * @param string|int $index <p>
 * Data index, must be string or integer.
 * </p>
 * @param bool $fallback
 * @return mixed the data located at the index or <b>NULL</b> on error. Strings, integers and binary data strings
 * are returned as corresponding PHP types, integer array is returned as PHP array. Complex types are
 * returned as <b>ResourceBundle</b> object.
 * @pure
 */
function resourcebundle_get(ResourceBundle $bundle, string|int $index, bool $fallback = true)
{
}

/**
 * (PHP &gt;= 5.3.2, PECL intl &gt;= 2.0.0)<br/>
 * Get number of elements in the bundle
 * @link https://php.net/manual/en/resourcebundle.count.php
 * @param ResourceBundle $bundle
 * @return int number of elements in the bundle.
 * @pure
 */
function resourcebundle_count(ResourceBundle $bundle): int
{
}

/**
 * (PHP &gt;= 5.3.2, PECL intl &gt;= 2.0.0)<br/>
 * Get supported locales
 * @link https://php.net/manual/en/resourcebundle.locales.php
 * @param string $bundle <p>
 * Path of ResourceBundle for which to get available locales, or
 * empty string for default locales list.
 * </p>
 * @return array|false the list of locales supported by the bundle.
 * @pure
 */
function resourcebundle_locales(string $bundle): array|false
{
}

/**
 * (PHP &gt;= 5.3.2, PECL intl &gt;= 2.0.0)<br/>
 * Get bundle's last error code.
 * @link https://php.net/manual/en/resourcebundle.geterrorcode.php
 * @param ResourceBundle $bundle
 * @return int error code from last bundle object call.
 */
function resourcebundle_get_error_code(ResourceBundle $bundle): int
{
}

/**
 * (PHP &gt;= 5.3.2, PECL intl &gt;= 2.0.0)<br/>
 * Get bundle's last error message.
 * @link https://php.net/manual/en/resourcebundle.geterrormessage.php
 * @param ResourceBundle $bundle
 * @return string error message from last bundle object's call.
 */
function resourcebundle_get_error_message(ResourceBundle $bundle): string
{
}

/**
 * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
 * Create a transliterator
 * @link https://php.net/manual/en/transliterator.create.php
 * @param string $id <p>
 * The id.
 * </p>
 * @param int $direction <p>
 * The direction, defaults to
 * Transliterator::FORWARD.
 * May also be set to
 * Transliterator::REVERSE.
 * </p>
 * @return Transliterator|null a <b>Transliterator</b> object on success,
 * or <b>NULL</b> on failure.
 * @since 5.4
 * @pure
 */
function transliterator_create(string $id, int $direction = 0): null|Transliterator
{
}

/**
 * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
 * Create transliterator from rules
 * @link https://php.net/manual/en/transliterator.createfromrules.php
 * @param string $rules <p>
 * The rules.
 * </p>
 * @param int $direction <p>
 * The direction, defaults to
 * Transliterator::FORWARD.
 * May also be set to
 * Transliterator::REVERSE.
 * </p>
 * @return Transliterator|null a <b>Transliterator</b> object on success,
 * or <b>NULL</b> on failure.
 * @since 5.4
 * @pure
 */
function transliterator_create_from_rules(string $rules, int $direction = 0): null|Transliterator
{
}

/**
 * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
 * Get transliterator IDs
 * @link https://php.net/manual/en/transliterator.listids.php
 * @return string[]|false An array of registered transliterator IDs on success,
 * or <b>FALSE</b> on failure.
 * @since 5.4
 * @pure
 */
function transliterator_list_ids(): array|false
{
}

/**
 * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
 * Create an inverse transliterator
 * @link https://php.net/manual/en/transliterator.createinverse.php
 * @param Transliterator $transliterator
 * @return Transliterator|null a <b>Transliterator</b> object on success,
 * or <b>NULL</b> on failure
 * @since 5.4
 * @pure
 */
function transliterator_create_inverse(Transliterator $transliterator): null|Transliterator
{
}

/**
 * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
 * Transliterate a string
 * @link https://php.net/manual/en/transliterator.transliterate.php
 * @param Transliterator|string $transliterator
 * @param string $string <p>
 * The string to be transformed.
 * </p>
 * @param int $start <p>
 * The start index (in UTF-16 code units) from which the string will start
 * to be transformed, inclusive. Indexing starts at 0. The text before will
 * be left as is.
 * </p>
 * @param int $end <p>
 * The end index (in UTF-16 code units) until which the string will be
 * transformed, exclusive. Indexing starts at 0. The text after will be
 * left as is.
 * </p>
 * @return string|false The transfomed string on success, or <b>FALSE</b> on failure.
 * @since 5.4
 * @pure
 */
function transliterator_transliterate(
    Transliterator|string $transliterator,
    string $string,
    int $start = 0,
    int $end = -1,
): string|false {
}

/**
 * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
 * Get last error code
 * @link https://php.net/manual/en/transliterator.geterrorcode.php
 * @param Transliterator $transliterator
 * @return int|false The error code on success,
 * or <b>FALSE</b> if none exists, or on failure.
 * @since 5.4
 */
function transliterator_get_error_code(Transliterator $transliterator): int|false
{
}

/**
 * (PHP &gt;= 5.4.0, PECL intl &gt;= 2.0.0)<br/>
 * Get last error message
 * @link https://php.net/manual/en/transliterator.geterrormessage.php
 * @param Transliterator $transliterator
 * @return string|false The error code on success,
 * or <b>FALSE</b> if none exists, or on failure.
 * @since 5.4
 */
function transliterator_get_error_message(Transliterator $transliterator): string|false
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get the last error code
 * @link https://php.net/manual/en/function.intl-get-error-code.php
 * @return int Error code returned by the last API function call.
 */
function intl_get_error_code(): int
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get description of the last error
 * @link https://php.net/manual/en/function.intl-get-error-message.php
 * @return string Description of an error occurred in the last API function call.
 */
function intl_get_error_message(): string
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Check whether the given error code indicates failure
 * @link https://php.net/manual/en/function.intl-is-failure.php
 * @param int $errorCode <p>
 * is a value that returned by functions:
 * <b>intl_get_error_code</b>,
 * <b>collator_get_error_code</b> .
 * </p>
 * @return bool <b>TRUE</b> if it the code indicates some failure, and <b>FALSE</b>
 * in case of success or a warning.
 * @pure
 */
function intl_is_failure(int $errorCode): bool
{
}

/**
 * (PHP 5 &gt;= 5.3.0, PECL intl &gt;= 1.0.0)<br/>
 * Get symbolic name for a given error code
 * @link https://php.net/manual/en/function.intl-error-name.php
 * @param int $errorCode <p>
 * ICU error code.
 * </p>
 * @return string The returned string will be the same as the name of the error code
 * constant.
 * @pure
 */
function intl_error_name(int $errorCode): string
{
}

/**
 * Gets the Decomposition_Mapping property for the given UTF-8 encoded code point
 *
 * @link https://www.php.net/manual/en/normalizer.getrawdecomposition.php
 *
 * @param string $string
 * @param int $form
 * @return string|null
 *
 * @since 7.3
 * @pure
 */
function normalizer_get_raw_decomposition(string $string, int $form = Normalizer::FORM_C): null|string
{
}

/**
 * @return IntlTimeZone
 * @pure
 */
function intltz_create_default(): IntlTimeZone
{
}

/**
 * @return IntlTimeZone
 * @pure
 */
function intltz_get_gmt(): IntlTimeZone
{
}

/**
 * @return IntlTimeZone
 * @pure
 */
function intltz_get_unknown(): IntlTimeZone
{
}

/**
 * @param int $type
 * @param null|string $region
 * @param null|int $rawOffset
 * @return IntlIterator|false
 * @pure
 */
function intltz_create_time_zone_id_enumeration(
    int $type,
    null|string $region = null,
    null|int $rawOffset = null,
): IntlIterator|false {
}

/**
 * @param string $timezoneId
 * @return string|false
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

    /* Methods */
    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Private constructor for disallowing instantiation
     */
    private function __construct() {}

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Create break iterator for boundaries of combining character sequences
     * @link https://secure.php.net/manual/en/intlbreakiterator.createcharacterinstance.php
     * @param string $locale
     * @return IntlBreakIterator|null
     */
    public static function createCharacterInstance(string|null $locale = null): null|IntlBreakIterator
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Create break iterator for boundaries of code points
     * @link https://secure.php.net/manual/en/intlbreakiterator.createcodepointinstance.php
     * @return IntlCodePointBreakIterator
     */
    public static function createCodePointInstance(): IntlCodePointBreakIterator
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Create break iterator for logically possible line breaks
     * @link https://secure.php.net/manual/en/intlbreakiterator.createlineinstance.php
     * @param string $locale [optional]
     * @return IntlBreakIterator|null
     */
    public static function createLineInstance(string|null $locale): null|IntlBreakIterator
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Create break iterator for sentence breaks
     * @link https://secure.php.net/manual/en/intlbreakiterator.createsentenceinstance.php
     * @param string $locale [optional]
     * @return IntlBreakIterator|null
     */
    public static function createSentenceInstance(string|null $locale): null|IntlBreakIterator
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Create break iterator for title-casing breaks
     * @link https://secure.php.net/manual/en/intlbreakiterator.createtitleinstance.php
     * @param string $locale [optional]
     * @return IntlBreakIterator|null
     */
    public static function createTitleInstance(string|null $locale): null|IntlBreakIterator
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Create break iterator for word breaks
     * @link https://secure.php.net/manual/en/intlbreakiterator.createwordinstance.php
     * @param string $locale [optional]
     * @return IntlBreakIterator|null
     */
    public static function createWordInstance(string|null $locale): null|IntlBreakIterator
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get index of current position
     * @link https://secure.php.net/manual/en/intlbreakiterator.current.php
     * @return int
     *
     * @pure
     */
    public function current(): int
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Set position to the first character in the text
     * @link https://secure.php.net/manual/en/intlbreakiterator.first.php
     */
    public function first(): int
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Advance the iterator to the first boundary following specified offset
     * @link https://secure.php.net/manual/en/intlbreakiterator.following.php
     * @param int $offset
     */
    public function following(int $offset): int
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get last error code on the object
     * @link https://secure.php.net/manual/en/intlbreakiterator.geterrorcode.php
     * @return int
     *
     * @pure
     */
    public function getErrorCode(): int
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get last error message on the object
     * @link https://secure.php.net/manual/en/intlbreakiterator.geterrormessage.php
     * @return string
     *
     * @pure
     */
    public function getErrorMessage(): string
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get the locale associated with the object
     * @link https://secure.php.net/manual/en/intlbreakiterator.getlocale.php
     * @param string $type
     *
     * @pure
     */
    public function getLocale(int $type): string|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Create iterator for navigating fragments between boundaries
     * @link https://secure.php.net/manual/en/intlbreakiterator.getpartsiterator.php
     * @param int $type [optional]
     * <p>
     * Optional key type. Possible values are:
     * </p><ul>
     * <li>
     * {@see IntlPartsIterator::KEY_SEQUENTIAL}
     * - The default. Sequentially increasing integers used as key.
     * </li>
     * <li>
     * {@see IntlPartsIterator::KEY_LEFT}
     * - Byte offset left of current part used as key.
     * </li>
     * <li>
     * {@see IntlPartsIterator::KEY_RIGHT}
     * - Byte offset right of current part used as key.
     * </li>
     * </ul>
     *
     * @pure
     */
    public function getPartsIterator($type = IntlPartsIterator::KEY_SEQUENTIAL): IntlPartsIterator
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get the text being scanned
     * @link https://secure.php.net/manual/en/intlbreakiterator.gettext.php
     *
     * @pure
     */
    public function getText(): null|string
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Tell whether an offset is a boundary's offset
     * @link https://secure.php.net/manual/en/intlbreakiterator.isboundary.php
     * @param int $offset
     *
     * @pure
     */
    public function isBoundary(int $offset): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Set the iterator position to index beyond the last character
     * @link https://secure.php.net/manual/en/intlbreakiterator.last.php
     * @return int
     */
    public function last(): int
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * @link https://secure.php.net/manual/en/intlbreakiterator.next.php
     * @param int $offset [optional]
     * @return int
     */
    public function next(null|int $offset = null): int
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * @link https://secure.php.net/manual/en/intlbreakiterator.preceding.php
     * @param int $offset
     */
    public function preceding(int $offset): int
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Set the iterator position to the boundary immediately before the current
     * @link https://secure.php.net/manual/en/intlbreakiterator.previous.php
     * @return int
     */
    public function previous(): int
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Set the text being scanned
     * @link https://secure.php.net/manual/en/intlbreakiterator.settext.php
     * @param string $text
     */
    public function setText(string $text): null|bool
    {
    }

    /**
     * @since 8.0
     * @return Iterator
     *
     * @pure
     */
    public function getIterator(): Iterator
    {
    }
}

class IntlRuleBasedBreakIterator extends IntlBreakIterator implements Traversable
{
    /* Methods */
    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * @link https://secure.php.net/manual/en/intlbreakiterator.construct.php
     * @param string $rules
     * @param string $compiled [optional]
     *
     * @pure
     */
    public function __construct(string $rules, bool $compiled = false) {}

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Create break iterator for boundaries of combining character sequences
     * @link https://secure.php.net/manual/en/intlbreakiterator.createcharacterinstance.php
     * @param string $locale
     * @return IntlRuleBasedBreakIterator
     */
    public static function createCharacterInstance($locale)
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Create break iterator for boundaries of code points
     * @link https://secure.php.net/manual/en/intlbreakiterator.createcodepointinstance.php
     * @return IntlRuleBasedBreakIterator
     */
    public static function createCodePointInstance()
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Create break iterator for logically possible line breaks
     * @link https://secure.php.net/manual/en/intlbreakiterator.createlineinstance.php
     * @param string $locale [optional]
     * @return IntlRuleBasedBreakIterator
     */
    public static function createLineInstance($locale)
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Create break iterator for sentence breaks
     * @link https://secure.php.net/manual/en/intlbreakiterator.createsentenceinstance.php
     * @param string $locale [optional]
     * @return IntlRuleBasedBreakIterator
     */
    public static function createSentenceInstance($locale)
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Create break iterator for title-casing breaks
     * @link https://secure.php.net/manual/en/intlbreakiterator.createtitleinstance.php
     * @param string $locale [optional]
     * @return IntlRuleBasedBreakIterator
     */
    public static function createTitleInstance($locale)
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Create break iterator for word breaks
     * @link https://secure.php.net/manual/en/intlbreakiterator.createwordinstance.php
     * @param string $locale [optional]
     * @return IntlRuleBasedBreakIterator
     */
    public static function createWordInstance($locale)
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * @link https://secure.php.net/manual/en/intlrulebasedbreakiterator.getbinaryrules.php
     * Get the binary form of compiled rules
     * @return string|false
     *
     * @pure
     */
    public function getBinaryRules(): string|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * @link https://secure.php.net/manual/en/intlrulebasedbreakiterator.getrules.php
     * Get the rule set used to create this object
     * @return string|false
     *
     * @pure
     */
    public function getRules(): string|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * @link https://secure.php.net/manual/en/intlrulebasedbreakiterator.getrulesstatus.php
     * Get the largest status value from the break rules that determined the current break position
     * @return int
     *
     * @pure
     */
    public function getRuleStatus(): int
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * @link https://secure.php.net/manual/en/intlrulebasedbreakiterator.getrulestatusvec.php
     * Get the status values from the break rules that determined the current break position
     * @return array|false
     *
     * @pure
     */
    public function getRuleStatusVec(): array|false
    {
    }
}

/**
 * @link https://www.php.net/manual/en/class.intlpartsiterator.php
 * @since 5.5
 */
class IntlPartsIterator extends IntlIterator implements Iterator
{
    public const KEY_SEQUENTIAL = 0;
    public const KEY_LEFT = 1;
    public const KEY_RIGHT = 2;

    /**
     * @return IntlBreakIterator
     *
     * @pure
     */
    public function getBreakIterator(): IntlBreakIterator
    {
    }

    /**
     * @since 8.1
     */
    public function getRuleStatus(): int
    {
    }
}

class IntlCodePointBreakIterator extends IntlBreakIterator implements Traversable
{
    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get last code point passed over after advancing or receding the iterator
     * @link https://secure.php.net/manual/en/intlcodepointbreakiterator.getlastcodepoint.php
     * @return int
     *
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

    /* Methods */
    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Create UConverter object
     * @link https://php.net/manual/en/uconverter.construct.php
     * @param string $destination_encoding
     * @param string $source_encoding
     *
     * @pure
     */
    public function __construct(string|null $destination_encoding = null, string|null $source_encoding = null) {}

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Convert string from one charset to anothe
     * @link https://php.net/manual/en/uconverter.convert.php
     * @param string $str
     * @param bool $reverse [optional]
     * @return string|false
     *
     * @pure
     */
    public function convert(string $str, bool $reverse = false): string|false
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Default "from" callback function
     * @link https://php.net/manual/en/uconverter.fromucallback.php
     * @param int $reason
     * @param string $source
     * @param string $codePoint
     * @param int &$error
     * @return array|string|int|null
     */
    public function fromUCallback(int $reason, array $source, int $codePoint, &$error): array|string|int|null
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get the aliases of the given name
     * @link https://php.net/manual/en/uconverter.getaliases.php
     * @param string $name
     * @return array|false|null
     */
    public static function getAliases(string $name): array|false|null
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get the available canonical converter names
     * @link https://php.net/manual/en/uconverter.getavailable.php
     * @return array
     */
    public static function getAvailable(): array
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get the destination encoding
     * @link https://php.net/manual/en/uconverter.getdestinationencoding.php
     * @return string|false|null
     *
     * @pure
     */
    public function getDestinationEncoding(): string|false|null
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get the destination converter type
     * @link https://php.net/manual/en/uconverter.getdestinationtype.php
     * @return int|false|null
     *
     * @pure
     */
    public function getDestinationType(): int|false|null
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get last error code on the object
     * @link https://php.net/manual/en/uconverter.geterrorcode.php
     * @return int
     *
     * @pure
     */
    public function getErrorCode(): int
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get last error message on the object
     * @link https://php.net/manual/en/uconverter.geterrormessage.php
     * @return string|null
     *
     * @pure
     */
    public function getErrorMessage(): null|string
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get the source encoding
     * @link https://php.net/manual/en/uconverter.getsourceencoding.php
     * @return string|false|null
     *
     * @pure
     */
    public function getSourceEncoding(): string|false|null
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get the source convertor type
     * @link https://php.net/manual/en/uconverter.getsourcetype.php
     * @return int|false|null
     *
     * @pure
     */
    public function getSourceType(): int|false|null
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get standards associated to converter names
     * @link https://php.net/manual/en/uconverter.getstandards.php
     * @return array|null
     *
     * @pure
     */
    public static function getStandards(): null|array
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get substitution chars
     * @link https://php.net/manual/en/uconverter.getsubstchars.php
     * @return string|false|null
     *
     * @pure
     */
    public function getSubstChars(): string|false|null
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Get string representation of the callback reason
     * @link https://php.net/manual/en/uconverter.reasontext.php
     * @param int $reason
     * @return string
     *
     * @pure
     */
    public static function reasonText(int $reason): string
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Set the destination encoding
     * @link https://php.net/manual/en/uconverter.setdestinationencoding.php
     * @param string $encoding
     * @return bool
     */
    public function setDestinationEncoding(string $encoding): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Set the source encoding
     * @link https://php.net/manual/en/uconverter.setsourceencoding.php
     * @param string $encoding
     * @return bool
     */
    public function setSourceEncoding(string $encoding): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Set the substitution chars
     * @link https://php.net/manual/en/uconverter.setsubstchars.php
     * @param string $chars
     * @return bool
     */
    public function setSubstChars(string $chars): bool
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Default "to" callback function
     * @link https://php.net/manual/en/uconverter.toucallback.php
     * @param int $reason
     * @param string $source
     * @param string $codeUnits
     * @param int &$error
     * @return array|string|int|null
     */
    public function toUCallback(int $reason, string $source, string $codeUnits, &$error): array|string|int|null
    {
    }

    /**
     * (PHP 5 &gt;=5.5.0)<br/>
     * Convert string from one charset to another
     * @link https://php.net/manual/en/uconverter.transcode.php
     * @param string $str
     * @param string $toEncoding
     * @param string $fromEncoding
     * @param array|null $options
     * @return string|false
     */
    public static function transcode(
        string $str,
        string $toEncoding,
        string $fromEncoding,
        array|null $options = null,
    ): string|false {
    }
}

// End of intl v.1.1.0
