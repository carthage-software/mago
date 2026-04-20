<?php

declare(strict_types=1);

/**
 * @param iterable<int, string> $iterable
 */
function consume(iterable $iterable): void
{
    if ($iterable instanceof Iterator) {
        while ($iterable->valid()) {
            $iterable->next();
        }

        return;
    }

    if ($iterable instanceof IteratorAggregate) {
        foreach ($iterable as $value) {
            echo $value;
        }

        return;
    }

    foreach ($iterable as $value) {
        echo $value;
    }
}
