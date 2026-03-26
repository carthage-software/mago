<?php

declare(strict_types=1);

class A
{
    public function __construct() {}

    public function getValue(): int
    {
        return 42;
    }
}

abstract class AbstractFoo
{
    // private(set) means: public read, private write
    // This property SHOULD be accessible from child classes for reading
    public private(set) A $a;

    public function __construct()
    {
        $this->a = new A();
    }
}

class Foo extends AbstractFoo
{
    public function __construct()
    {
        parent::__construct();
    }

    public function test(): int
    {
        // $this->a should be accessible here because read visibility is public
        return $this->a->getValue();
    }
}

$foo = new Foo();
echo $foo->test();
