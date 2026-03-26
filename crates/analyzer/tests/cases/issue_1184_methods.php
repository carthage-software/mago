<?php

declare(strict_types=1);

class BaseClass
{
    public function bar(): string
    {
        return 'hello';
    }
}

/**
 * @method string bar()
 */
trait MyTrait
{
}

class MiddleClass extends BaseClass
{
    use MyTrait;
}

class ChildClass extends MiddleClass
{
    public function test(): string
    {
        return $this->bar();
    }
}
