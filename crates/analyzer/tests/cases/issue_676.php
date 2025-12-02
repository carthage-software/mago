<?php

declare(strict_types=1);

interface I
{
    /**
     * @template S1 of scalar
     * @template S2 of S1
     * @param S1 $s1
     * @param S2 $s2
     */
    function f(mixed $s1, mixed $s2): void;
}

function test_valid(I $i, bool $b, int $int, string $str): void
{
    // S1 inferred as bool, S2's constraint resolves to bool
    $i->f($b, false); // OK: false is bool
    $i->f($b, true); // OK: true is bool
    $i->f($b, $b); // OK: bool is bool

    // S1 inferred as int, S2's constraint resolves to int
    $i->f($int, 0); // OK: 0 is int
    $i->f($int, $int); // OK: int is int

    // S1 inferred as string, S2's constraint resolves to string
    $i->f($str, 'b'); // OK: 'b' is string
    $i->f($str, $str); // OK: string is string
}

function test_invalid(I $i, bool $b, int $int, string $str): void
{
    // @mago-expect analysis:template-constraint-violation
    // @mago-expect analysis:invalid-argument
    $i->f($b, 'str');
    // @mago-expect analysis:template-constraint-violation
    // @mago-expect analysis:invalid-argument
    $i->f($int, 'str');
    // @mago-expect analysis:template-constraint-violation
    // @mago-expect analysis:invalid-argument
    $i->f($str, 1);
}
