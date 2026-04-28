<?php

declare(strict_types=1);

/**
 * @template K
 * @template V
 *
 * @mago-expect analysis:unused-template-parameter
 */
abstract class GenPartialBase
{
    /** @return list<V> */
    abstract public function values(): array;
}

/**
 * @template V
 *
 * @extends GenPartialBase<int, V>
 */
abstract class GenPartialIntK extends GenPartialBase
{
}

/**
 * @extends GenPartialIntK<string>
 */
final class GenPartialFinal extends GenPartialIntK
{
    public function values(): array
    {
        return ['a', 'b'];
    }
}

function take_str_final(string $s): void
{
}

foreach ((new GenPartialFinal())->values() as $v) {
    take_str_final($v);
}
