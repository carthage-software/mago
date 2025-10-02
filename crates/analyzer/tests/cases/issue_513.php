<?php

function foo(mixed $x): void
{
    // @mago-expect analysis:mixed-argument,mixed-assignment
    $current = current($x);
    if ($current === 123)
        die('Was true');
}

/**
 * @template T
 *
 * @return T|false
 */
function a(): mixed
{
    // `T` is `mixed`, so `true` is acceptable.
    // This is not allowed by Psalm or PHPStan, but it does not make sense to disallow it.
    return true;
}

// `T` is not inferred, so we set the lower bound to it's upper bound, which is `mixed`.
// @mago-expect analysis:mixed-assignment
$a = a();
if ($a === '') {
    // `$a` should have type `string('')` now, so a call to `strlen` is valid.
    echo 'It is an empty string, we know length is ' . strlen($a);
}

foo([123]);
