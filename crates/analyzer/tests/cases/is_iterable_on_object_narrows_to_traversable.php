<?php

declare(strict_types=1);

class IsIterableBox {}

function is_iterable_on_object(object $v): int
{
    if (is_iterable($v)) {
        return iterator_count($v);
    }

    return 0;
}

function is_iterable_on_open_class(IsIterableBox $b): int
{
    if (is_iterable($b)) {
        return iterator_count($b);
    }

    return 0;
}
