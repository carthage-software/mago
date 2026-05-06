<?php

declare(strict_types=1);

// isset() on a possibly-undefined variable should NOT trigger
// possibly-undefined-variable — that's exactly what isset() checks.
function test_isset_suppresses_possibly_undefined(bool $cond): void
{
    if ($cond) {
        $x = 42;
    }

    if (isset($x)) {
        echo $x;
    }
}

// Same for empty().
function test_empty_suppresses_possibly_undefined(bool $cond): void
{
    if ($cond) {
        $y = 'hello';
    }

    if (!empty($y)) {
        echo $y;
    }
}
