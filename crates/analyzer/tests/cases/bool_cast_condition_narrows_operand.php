<?php

declare(strict_types=1);

final class Foo {}

/** @param 0 $n */
function takes_zero(int $n): void
{
    echo $n;
}

function takes_foo(Foo $foo): void
{
    var_dump($foo);
}

function narrow(?Foo $foo): void
{
    $n = rand(0, 6);
    if (!(bool) $n) {
        takes_zero($n);
    }

    if ((bool) $foo) {
        takes_foo($foo);
    }

    while ((bool) ($a = rand(0, 6))) {
        echo 'again';
    }

    takes_zero($a);
}
