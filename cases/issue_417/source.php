<?php

declare(strict_types=1);

use function count as some_count;

class Handler
{
    /**
     * @param array<int, int> $rawData
     */
    function x(array $rawData): int|false
    {
        if (\count($rawData) !== 1) {
            return false;
        }

        $item = \end($rawData);
        return self::processItem($item);
    }

    /**
     * @param array<int, int> $rawData
     */
    public static function y(array $rawData): int|false
    {
        if (some_count($rawData) !== 1) {
            return false;
        }

        $item = \end($rawData);
        return self::processItem($item);
    }

    static function processItem(int $item): int
    {
        return $item * 2;
    }
}
