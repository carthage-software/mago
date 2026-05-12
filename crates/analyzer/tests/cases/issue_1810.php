<?php

declare(strict_types=1);

class Test1810
{
    private const DATA = [
        'one' => 1,
        'two' => 2,
    ];

    /**
     * @template K of key-of<self::DATA>
     *
     * @param K $key
     *
     * @return self::DATA[K]
     */
    public function get(string $key): int
    {
        return self::DATA[$key];
    }
}
