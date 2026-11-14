<?php

declare(strict_types=1);

class Root {}

class Base extends Root
{
    protected string $prop = '';

    protected static int $count = 0;

    protected function name(): string
    {
        return 'base';
    }
}

final class Redeclares extends Base
{
    protected string $prop = 'redeclared';

    protected static int $count = 1;

    protected function name(): string
    {
        return 'redeclared';
    }
}

final class Sibling extends Base
{
    public function read(Redeclares $other): string
    {
        return $other->prop . $other->name() . Redeclares::$count;
    }

    public function write(Redeclares $other): void
    {
        $other->prop = 'written';
    }
}

final class Unrelated extends Root
{
    public function read(Redeclares $other): string
    {
        // @mago-expect analysis:invalid-method-access
        return $other->name();
    }
}
