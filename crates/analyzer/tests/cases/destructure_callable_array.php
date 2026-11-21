<?php

declare(strict_types=1);

/**
 * @param array{0: object, 1: string} $callback
 */
function destructure_callable_array(array $callback): void
{
    if (is_callable($callback)) {
        [$object, $method] = $callback;
        var_dump($object, $method);
    }
}
