<?php

declare(strict_types=1);

function take_int(int $value): void
{
}

function ternary(bool $flag): void
{
    $result = $flag ? ($assigned = 1) : null;
    take_int($assigned); // @mago-expect analysis:possibly-undefined-variable
    echo $result ?? 0;
}

function short_ternary(bool $flag): void
{
    $result = $flag ?: ($assigned = 1);
    take_int($assigned); // @mago-expect analysis:possibly-undefined-variable
    echo (int) $result;
}

function match_arm(int $value): void
{
    $result = match ($value) {
        1 => ($assigned = 1),
        default => null,
    };
    take_int($assigned); // @mago-expect analysis:possibly-undefined-variable
    echo $result ?? 0;
}

function every_match_arm(int $value): int
{
    $result = match ($value) {
        1 => ($assigned = 1),
        default => ($assigned = 2),
    };

    return $assigned + $result;
}
