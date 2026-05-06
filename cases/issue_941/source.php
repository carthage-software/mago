<?php

declare(strict_types=1);

abstract class Base
{
    protected string $value = 'test';

    public function test()
    {
        assert($this->value === '123', 'Value must be 123.');
    }
}

final class Child extends Base
{
    protected string $value = '123';
}
