<?php

/**
 * @var int
 */
const CAL_GREGORIAN = UNKNOWN;

/**
 * @var int
 */
const CAL_JULIAN = UNKNOWN;

/**
 * @var int
 */
const CAL_JEWISH = UNKNOWN;

/**
 * @var int
 */
const CAL_FRENCH = UNKNOWN;

/**
 * @var int
 */
const CAL_NUM_CALS = UNKNOWN;

/**
 * @var int
 */
const CAL_DOW_DAYNO = UNKNOWN;

/**
 * @var int
 */
const CAL_DOW_SHORT = UNKNOWN;

/**
 * @var int
 */
const CAL_DOW_LONG = UNKNOWN;

/**
 * @var int
 */
const CAL_MONTH_GREGORIAN_SHORT = UNKNOWN;

/**
 * @var int
 */
const CAL_MONTH_GREGORIAN_LONG = UNKNOWN;

/**
 * @var int
 */
const CAL_MONTH_JULIAN_SHORT = UNKNOWN;

/**
 * @var int
 */
const CAL_MONTH_JULIAN_LONG = UNKNOWN;

/**
 * @var int
 */
const CAL_MONTH_JEWISH = UNKNOWN;

/**
 * @var int
 */
const CAL_MONTH_FRENCH = UNKNOWN;

/**
 * @var int
 */
const CAL_EASTER_DEFAULT = UNKNOWN;

/**
 * @var int
 */
const CAL_EASTER_ROMAN = UNKNOWN;

/**
 * @var int
 */
const CAL_EASTER_ALWAYS_GREGORIAN = UNKNOWN;

/**
 * @var int
 */
const CAL_EASTER_ALWAYS_JULIAN = UNKNOWN;

/**
 * @var int
 */
const CAL_JEWISH_ADD_ALAFIM_GERESH = UNKNOWN;

/**
 * @var int
 */
const CAL_JEWISH_ADD_ALAFIM = UNKNOWN;

/**
 * @var int
 */
const CAL_JEWISH_ADD_GERESHAYIM = UNKNOWN;

function jdtogregorian(int $julian_day): string
{
}

function gregoriantojd(int $month, int $day, int $year): int
{
}

function jdtojulian(int $julian_day): string
{
}

function juliantojd(int $month, int $day, int $year): int
{
}

function jdtojewish(int $julian_day, bool $hebrew = false, int $flags = 0): string
{
}

function jewishtojd(int $month, int $day, int $year): int
{
}

function jdtofrench(int $julian_day): string
{
}

function frenchtojd(int $month, int $day, int $year): int
{
}

function jddayofweek(int $julian_day, int $mode = CAL_DOW_DAYNO): string|int
{
}

function jdmonthname(int $julian_day, int $mode): string
{
}

function easter_date(null|int $year, int $mode = CAL_EASTER_DEFAULT): int
{
}

/**
 * @param positive-int|null $year
 */
function easter_days(null|int $year, int $mode = CAL_EASTER_DEFAULT): int
{
}

function unixtojd(null|int $timestamp = null): int|false
{
}

function jdtounix(int $julian_day): int
{
}

function cal_to_jd(int $calendar, int $month, int $day, int $year): int
{
}

/**
 * @return array{
 *     date: string,
 *     month: int,
 *     day: int,
 *     year: int,
 *     dow: int,
 *     abbrevdayname: string,
 *     dayname: string,
 *     abbrevmonth: string,
 *     monthname: string
 * }
 */
function cal_from_jd(int $julian_day, int $calendar): array
{
}

function cal_days_in_month(int $calendar, int $month, int $year): int
{
}

/**
 * @return array{
 *     months: array<string>,
 *     abbrevmonths: array<string>,
 *     maxdaysinmonth: int,
 *     calname: string,
 *     calsymbol: string
 * }
 */
function cal_info(int $calendar = -1): array
{
}
