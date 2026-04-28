<?php

declare(strict_types=1);

/**
 * @template K
 * @template V
 */
abstract class GenAssoc
{
    /** @return list<K> */
    abstract public function keys(): array;

    /** @return list<V> */
    abstract public function values(): array;
}

/**
 * @extends GenAssoc<int, string>
 */
final class GenIntStrAssoc extends GenAssoc
{
    public function keys(): array
    {
        return [1, 2];
    }

    public function values(): array
    {
        return ['a', 'b'];
    }
}

function takes_int_only_x(int $n): void
{
}

function takes_str_only_x(string $s): void
{
}

$a = new GenIntStrAssoc();
foreach ($a->keys() as $k) {
    takes_int_only_x($k);
}
foreach ($a->values() as $v) {
    takes_str_only_x($v);
}
