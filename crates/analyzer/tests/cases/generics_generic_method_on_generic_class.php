<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenCls
{
    /** @param T $v */
    public function __construct(public mixed $v)
    {
    }

    /**
     * @template S
     *
     * @param S $other
     *
     * @return array{T, S}
     */
    public function pair(mixed $other): array
    {
        return [$this->v, $other];
    }
}

function take_int_str(int $a, string $b): void
{
}

$res = (new GenCls(1))->pair('hi');
take_int_str($res[0], $res[1]);
