<?php

declare(strict_types=1);

final class Box
{
    public int $count = 0;

    public function take(int $value): int
    {
        return $value;
    }

    public function count(): int
    {
        return $this->count;
    }
}

function arguments(?Box $box): ?int
{
    $first = $box?->take($box->count);
    $second = $box?->take($box->count());

    return $first ?? $second;
}

function receiver_stays_nullable(?Box $box): int
{
    $box?->take($box->count);

    return $box->count; // @mago-expect analysis:possibly-null-property-access,nullable-return-statement,invalid-return-statement
}
