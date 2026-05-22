<?php

declare(strict_types=1);

namespace App;

abstract class Foo
{
    /**
     * @var array<int, string>
     */
    protected array $commands = [];
}

class Bar extends Foo
{
    protected array $commands = [
        SomeCommand::class,
    ];
}

class SomeCommand {}
