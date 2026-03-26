<?php

/**
 * @param (Closure(): Generator<non-negative-int, true, mixed, void>) $_fn
 */
function takes_yield_value(Closure $_fn): void {}

/**
 * @param (Closure(): Generator<non-negative-int, null, mixed, void>) $_fn
 */
function takes_yield_bare(Closure $_fn): void {}

/**
 * @param (Closure(): Generator<'key', 'value', mixed, void>) $_fn
 */
function takes_yield_pair(Closure $_fn): void {}

/**
 * @param (Closure(): Generator<non-negative-int, true, mixed, 'hello'>) $_fn
 */
function takes_yield_with_return(Closure $_fn): void {}

/**
 * @param (Closure(): Generator<non-negative-int, 1|2|3, mixed, void>) $_fn
 */
function takes_yield_multi(Closure $_fn): void {}

/**
 * @param (Closure(): Generator<non-negative-int|'x', 1|2, mixed, void>) $_fn
 */
function takes_yield_mixed_keys(Closure $_fn): void {}

/**
 * @param (Closure(): Generator<non-negative-int, 'hello', mixed, mixed>) $_fn
 */
function takes_arrow_yield(Closure $_fn): void {}

/**
 * @param (Closure(bool): Generator<non-negative-int, 'yes'|'no', mixed, void>) $_fn
 */
function takes_yield_conditional(Closure $_fn): void {}

/**
 * @param (Closure(): Generator<non-negative-int, 1, mixed, void>) $_fn
 */
function takes_yield_bare_return(Closure $_fn): void {}

takes_yield_value(function () {
    yield true;
});

takes_yield_bare(function () {
    yield;
});

takes_yield_pair(function () {
    yield 'key' => 'value';
});

takes_yield_with_return(function () {
    yield true;
    return 'hello';
});

takes_yield_multi(function () {
    yield 1;
    yield 2;
    yield 3;
});

takes_yield_mixed_keys(function () {
    yield 1;
    yield 'x' => 2;
});

takes_arrow_yield(fn() => yield 'hello');

takes_yield_conditional(function (bool $flag) {
    if ($flag) {
        yield 'yes';
    } else {
        yield 'no';
    }
});

takes_yield_bare_return(function () {
    yield 1;
    return;
});

takes_yield_bare(function () {
    yield;
    yield;
    yield;
});

/**
 * @param (Closure(): Generator<'a'|'b', 1|2, mixed, void>) $_fn
 */
function takes_yield_from_gen(Closure $_fn): void {}

/** @return Generator<'a'|'b', 1|2, mixed, void> */
function inner_gen(): Generator
{
    yield 'a' => 1;
    yield 'b' => 2;
}

takes_yield_from_gen(function () {
    yield from inner_gen();
});

/**
 * @param (Closure(): Generator<'a'|'b', 1|2, mixed, 42>) $_fn
 */
function takes_yield_from_with_return(Closure $_fn): void {}

takes_yield_from_with_return(function () {
    yield from inner_gen();
    return 42;
});
