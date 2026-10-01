<?php

declare(strict_types=1);

/** @param truthy-string $s */
function issue2401_takes_truthy(string $s): void
{
    echo $s;
}

/** @param lowercase-string $s */
function issue2401_takes_lowercase(string $s): void
{
    echo $s;
}

/** @param numeric-string $s */
function issue2401_takes_numeric(string $s): void
{
    echo $s;
}

/** @return lowercase-string&non-falsy-string */
function issue2401_lowercase_non_falsy(): string
{
    return 'abc';
}

/** @return truthy-string&lowercase-string */
function issue2401_truthy_lowercase(): string
{
    return 'abc';
}

/** @return lowercase-string&numeric-string */
function issue2401_lowercase_numeric(): string
{
    return '1';
}

/** @return non-empty-string&numeric-string */
function issue2401_non_empty_numeric(): string
{
    return '1';
}

/** @return non-falsy-string&numeric-string */
function issue2401_non_falsy_numeric(): string
{
    return '1';
}

/** @return lowercase-string&non-falsy-string */
function issue2401_lowercase_non_falsy_rejects_uppercase(): string
{
    /** @mago-expect analysis:invalid-return-statement */
    return 'ABC';
}

/** @return lowercase-string&non-falsy-string */
function issue2401_lowercase_non_falsy_rejects_zero(): string
{
    /** @mago-expect analysis:invalid-return-statement */
    return '0';
}

/**
 * @return (
 *     ( $prefix is ''|numeric-string ? numeric-string : string )
 *     & non-falsy-string
 *     & ( $prefix is lowercase-string ? lowercase-string : string )
 * )
 */
function issue2401_unique_id(string $prefix = ''): string
{
    return $prefix . 'x';
}

issue2401_takes_truthy(issue2401_lowercase_non_falsy());
issue2401_takes_lowercase(issue2401_lowercase_non_falsy());
issue2401_takes_truthy(issue2401_truthy_lowercase());
issue2401_takes_lowercase(issue2401_truthy_lowercase());
issue2401_takes_lowercase(issue2401_lowercase_numeric());
issue2401_takes_numeric(issue2401_lowercase_numeric());
issue2401_takes_numeric(issue2401_non_empty_numeric());
issue2401_takes_truthy(issue2401_non_falsy_numeric());
issue2401_takes_numeric(issue2401_non_falsy_numeric());

issue2401_takes_truthy(issue2401_unique_id('abc'));
issue2401_takes_lowercase(issue2401_unique_id('abc'));
issue2401_takes_numeric(issue2401_unique_id(''));
issue2401_takes_truthy(issue2401_unique_id('ABC'));
/** @mago-expect analysis:possibly-invalid-argument */
issue2401_takes_lowercase(issue2401_unique_id('ABC'));
