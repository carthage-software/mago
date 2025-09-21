<?php

/**
 * @param list{true, false} $arr
 */
function usingtf(array $arr): void
{
    printf("Count: %d\n", count($arr));
}

/**
 * @param list{false, true} $arr
 */
function usingft(array $arr): void
{
    printf("Count: %d\n", count($arr));
}

/**
 * @param list{true, true} $arr
 */
function usingtt(array $arr): void
{
    printf("Count: %d\n", count($arr));
}

/**
 * @param list{false, false} $arr
 */
function usingff(array $arr): void
{
    printf("Count: %d\n", count($arr));
}

/**
 * @param list{false, bool}|list{bool, false} $arr
 */
function using1(array $arr): void
{
    printf("Count: %d\n", count($arr));
}

/**
 * @param list{true, false}|list{false, true} $arr
 */
function using2(array $arr): void
{
    printf("Count: %d\n", count($arr));
}

/**
 * @param list{true, false} $arr
 */
function using3(array $arr): void
{
    printf("Count: %d\n", count($arr));
}

/**
 * @param list{false, true} $arr
 */
function using4(array $arr): void
{
    printf("Count: %d\n", count($arr));
}

/**
 * @param list{true, true} $arr
 */
function using5(array $arr): void
{
    printf("Count: %d\n", count($arr));
}

/**
 * @param list{false, false} $arr
 */
function using6(array $arr): void
{
    printf("Count: %d\n", count($arr));
}

/**
 * @param list{string} $arr
 */
function usingString(array $arr): void
{
    printf("Count: %d\n", count($arr));
}

function booleans1_if(bool $a, bool $b): void
{
    $arr = [$a, $b];
    if ($arr === [true, true]) {
        usingtt($arr);
    } else {
        if ($arr === [false, true]) {
            usingft($arr);
        } else {
            if ($arr === [true, false]) {
                usingtf($arr);
            } else {
                usingff($arr);
            }
        }
    }
}

function booleans2_if(bool $a, bool $b): void
{
    $arr = [$a, $b];
    if ($arr === [true, true]) {
        using5($arr);
    } else {
        using1($arr);

        if ($arr === [false, false]) {
            using6($arr);
        } else {
            using2($arr);

            if ($arr === [true, false]) {
                using3($arr);
            } else {
                using4($arr);
            }
        }
    }
}

function test_null_string_subtraction(null|string $val): void
{
    $arr = [$val];
    if ($arr === [null]) {
        // Do nothing, we've excluded null case
    } else {
        usingString($arr); // $arr is now list{string}
    }
}
