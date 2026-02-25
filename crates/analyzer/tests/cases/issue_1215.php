<?php

declare(strict_types=1);

class TestType {}

class Test
{
    private const array DEFAULT_CONFIG = [
        'someKey' => 123,
        'title' => 'Title',
        'slug' => 'default',
    ];

    /** @var array<string, TestType> */
    public const array TEST1 = [
        'test1' => [
            ...self::DEFAULT_CONFIG,
            'title' => 'Test Title 1',
            'slug' => 'default-1',
        ],
    ];

    /** @var array<string, TestType> */
    public const array TEST2 = [
        'test2' => [
            ...self::DEFAULT_CONFIG,
            'title' => 'Test Title 2',
            'slug' => 'default-2',
            'extra' => true,
        ],
    ];
}
