<?php

declare(strict_types=1);

final class Fin {}

/** @param iterable<int, string> $i */
function check(Fin $x, iterable $i): void
{
    // @mago-expect analysis:impossible-condition
    if ($i instanceof Fin) {
        echo 'never';
    }

    // @mago-expect analysis:impossible-condition,redundant-comparison
    if ($x === $i) {
        echo 'never';
    }

    if ($i instanceof ArrayIterator) {
        echo 'maybe';
    }
}
