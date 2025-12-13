<?php

declare(strict_types=1);

final class Package
{
    private const string EMPTY_VALUE = 'k. A.';

    /** @var numeric-string|self::EMPTY_VALUE */
    private string $quantity1;

    /** @var numeric-string|'Empty' */
    private string $quantity2;

    public function __construct(int|null $quantity1, int|null $quantity2)
    {
        $this->quantity1 = $quantity1 !== null ? (string) $quantity1 : self::EMPTY_VALUE;
        $this->quantity2 = $quantity2 !== null ? (string) $quantity2 : 'Empty';
    }

    public function quantity1(): int|null
    {
        return $this->quantity1 !== self::EMPTY_VALUE ? (int) $this->quantity1 : null;
    }

    public function quantity2(): int|null
    {
        return $this->quantity2 !== 'Empty' ? (int) $this->quantity2 : null;
    }
}
