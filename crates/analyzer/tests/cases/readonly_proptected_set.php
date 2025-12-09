<?php

class A
{
    public readonly string $foo;

    public function __construct() {
        $this->foo = 'initial';
    }

    /**
     * @mago-expect analysis:invalid-property-write - Cannot modify a readonly property after initialization
     */
    public function foo(): void
    {
        $this->foo = 'baz';
    }
}

class B extends A
{
    /**
     * @mago-expect analysis:invalid-property-write - Cannot modify a readonly property after initialization
     */
    public function foo(): void
    {
        $this->foo = 'qux';
    }
}

/**
 * @mago-expect analysis:invalid-property-write - Cannot write to protected property `$foo` on class `A`.
 */
function example(): void
{
    $a = new A();
    $a->foo = 'bar';
}
