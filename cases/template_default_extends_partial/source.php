<?php

declare(strict_types=1);

/**
 * @template TKey of array-key
 * @template TValue = string
 */
abstract class Box
{
    /**
     * @param TKey $key
     * @param TValue $value
     */
    abstract public function set($key, $value): void;
}

/**
 * @extends Box<int>
 */
final class IntBox extends Box
{
    public function set($key, $value): void
    {
        // TValue defaults to string, so $value is string here.
        echo $key + 1;
        echo strlen($value);
    }
}
