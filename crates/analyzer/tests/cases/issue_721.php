<?php

/**
 * @param class-string $modelClass
 */
function create(string $modelClass, string $value): mixed
{
    $callable = [$modelClass, 'fromString'];

    if (is_callable($callable)) {
        return $callable($value);
    }

    return null;
}
