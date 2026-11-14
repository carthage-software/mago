<?php

declare(strict_types=1);

/**
 * @return Closure(int): string
 */
function returned_closure_one(): Closure
{
    return fn($number) => 'n' . $number;
}

/** @return Closure(int): Closure(string): string */
function returned_closure_two(): Closure
{
    return fn($number) => fn($text) => $text . $number;
}

/** @return Closure(int): Closure(string): int */
function returned_closure_wrong(): Closure
{
    // @mago-expect analysis:invalid-return-statement
    return fn($number) => fn($text) => $text;
}

/** @return Closure(int): string */
function returned_closure_statement(): Closure
{
    return function ($number) {
        return 'n' . $number;
    };
}

/** @param Closure(int): (Closure(string): string) $build */
function returned_closure_run(Closure $build): string
{
    return $build(1)('a');
}

function returned_closure_argument(): string
{
    return returned_closure_run(fn($number) => fn($text) => $text . $number);
}
