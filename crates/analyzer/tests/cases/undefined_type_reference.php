<?php

// @mago-expect analysis:non-existent-class-like
function test(Bar $_x): void
{
}

// @mago-expect analysis:non-existent-class-like,missing-return-statement
function test2(): Baz
{
}

// @mago-expect analysis:non-existent-class-like,missing-return-statement
function test3(): stdClass&Baz
{
}

// @mago-expect analysis:non-existent-class-like
function test4(): string|(stdClass&Baz)
{
    return 'hello';
}

// @mago-expect analysis:non-existent-class-like
class Example
{
    public ?Qux $prop = null;
}

// @mago-expect analysis:non-existent-class-like
class OtherSample
{
    /**
     * @param Undefined $_x
     */
    public function test3($_x): void
    {
    }
}

function valid(string $_x, int $_y): stdClass
{
    return new stdClass();
}

class Valid
{
    public string $name = '';
    public ?int $count = null;
}

// @mago-expect analysis:non-existent-class-like
function test5(?Unknown $_x): void
{
}

class WithMethod
{
    // @mago-expect analysis:non-existent-class-like,missing-return-statement
    public function method(Invalid $_p): Missing
    {
    }
}
