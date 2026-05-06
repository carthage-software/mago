<?php

declare(strict_types=1);

/** @return Closure(int): int */
function callables_make_fact(): Closure
{
    /** @var null|Closure(int): int $self */
    $self = null;
    $self = function (int $n) use (&$self): int {
        if ($n <= 1) {
            return 1;
        }
        /** @var Closure(int): int $self */
        return $n * $self($n - 1);
    };
    return $self;
}

$f = callables_make_fact();
echo $f(4);
