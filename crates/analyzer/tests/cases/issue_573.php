<?php

declare(strict_types=1);

class Foo
{
    /** @var array<string, string> */
    private const ORDER_BY = [
        'id' => 'a.id',
    ];

    /** @var array<string, string> */
    private static array $DIRECTION = [
        'asc' => 'ASC',
        'desc' => 'DESC',
    ];

    private function bar(null|string $order, null|string $direction): void
    {
        if (isset(self::ORDER_BY[$order], self::$DIRECTION[$direction])) {
            $order = self::ORDER_BY[$order];
            $direction = self::$DIRECTION[$direction];
        }
    }
}
