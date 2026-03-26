<?php

interface Foo
{
    public function foo(): void;
}

interface Bar
{
    public function bar(): void;
}

/**
 * Test @var with spaced intersection type on property.
 */
class TestClass
{
    /** @var Foo & Bar Spaced intersection on property */
    private object $intersectionProp;

    /**
     * @mago-expect analysis:write-only-property
     * @var int | string Spaced union on property
     */
    private int|string $unionProp;

    public function __construct(Foo&Bar $value, int|string $union)
    {
        $this->intersectionProp = $value;
        $this->unionProp = $union;
    }

    public function useIntersection(): void
    {
        $this->intersectionProp->foo();
        $this->intersectionProp->bar();
    }
}

function getObject(): object
{
    return new class implements Foo, Bar {
        public function foo(): void
        {
        }

        public function bar(): void
        {
        }
    };
}

/**
 * Test @var with spaced intersection type on local variable.
 */
function testSpacedIntersectionVar(): void
{
    /** @var Foo & Bar $x Spaced intersection */
    $x = getObject();
    $x->foo();
    $x->bar();
}

/**
 * Test @var with spaced union type on local variable.
 */
function testSpacedUnionVar(): void
{
    /** @var int | string $y Spaced union */
    $y = rand(0, 1) ? 42 : 'hello';
    echo $y;
}
