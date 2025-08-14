<?php

class A
{
    /** @var int */
    private $foo;

    public function __construct(int &$foo)
    {
        $this->foo = &$foo;
    }
}

class B
{
    /** @var string */
    private $bar;

    public function __construct(string &$bar)
    {
        $this->bar = &$bar;
    }
}

function get_bool(): bool
{
    return get_bool();
}

/**
 * @mago-expect analysis:conflicting-reference-constraint
 */
function main(): void
{
    if (get_bool()) {
        $v = 5;
        $c = new A($v); // $v is constrained to an int
    } else {
        $v = 'hello';
        $c = new B($v); // $v is constrained to a string
    }

    $v = 8;
}
