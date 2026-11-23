<?php

declare(strict_types=1);

/**
 * @param Closure(mixed...): mixed $callback
 * @param list<mixed> $arguments
 */
function spreading_empty_array_passes_no_arguments(Closure $callback, array $arguments): mixed
{
    $callback(...[]);

    return $callback(...$arguments, ...($arguments = []));
}
