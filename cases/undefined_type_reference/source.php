<?php

function test(Bar $_x): void {}

function test2(): Baz {}

function test3(): stdClass&Baz {}

function test4(): string|(stdClass&Baz)
{
    return 'hello';
}

class Example
{
    public ?Qux $prop = null;
}

class OtherSample
{
    /**
     * @param Undefined $_x
     */
    public function test3($_x): void {}
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

function test5(?Unknown $_x): void {}

class WithMethod
{
    public function method(Invalid $_p): Missing {}
}

/** @var null|NonExistingClass */
$x = null;

function test_instanceof(mixed $x): void
{
    if ($x instanceof NonExistingClass) {
    }

    if ($x instanceof stdClass) {
    }
}
