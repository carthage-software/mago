<?php declare(strict_types=1);

namespace Reproduction;

interface One {}

interface Two extends One
{
    public function x(): string;
}

class A
{
    /** @return One */
    public function get(): One
    {
        return new class implements One {};
    }
}

class B extends A
{
    /** @inheritDoc */
    public function get(): Two
    {
        return new class implements Two {
            public function x(): string
            {
                return 'foo';
            }
        };
    }
}

$b = new B();
echo $b->get()->x();
