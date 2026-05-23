<?php

declare(strict_types=1);

/**
 * @template Tk of array-key
 * @template Tv
 *
 * @param iterable<Tk> $keys
 * @param (Closure(Tk): Tv) $value_func
 *
 * @return array<Tk, Tv>
 */
function callable_template_from_keys(iterable $keys, Closure $value_func): array
{
    $result = [];
    foreach ($keys as $key) {
        $result[$key] = $value_func($key);
    }

    return $result;
}

callable_template_from_keys(
    ['sm', 'md', 'lg'],
    /**
     * @param 'sm'|'md'|'lg' $size
     *
     * @return 576|768|992
     */
    fn(string $size): int => match ($size) {
        'sm' => 576,
        'md' => 768,
        'lg' => 992,
    },
);
