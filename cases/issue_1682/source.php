<?php

declare(strict_types=1);

function test(string $s = false): void
{
    echo $s;
}

// `null` default on a non-nullable type is legal (implicitly nullable) on PHP < 9.0.
function implicit_null_default(string $s = null): void
{
    unset($s);
}

function ok_default(string $s = 'default'): void
{
    echo $s;
}

function ok_nullable(?string $s = null): void
{
    echo $s ?? '';
}

function ok_union(string|int $v = 0): void
{
    echo $v;
}
