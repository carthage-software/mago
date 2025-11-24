<?php

declare(strict_types=1);

class Foo
{
}

// Test 1: @method refining inherited method return type (should work without __call)
class Root
{
    /** @return object */
    public function getFoo()
    {
        return new Foo();
    }

    /** @return object */
    public function getBar()
    {
        return new Foo();
    }
}

/**
 * @method Foo getFoo()  Refines return type from object to Foo
 * @method Foo getBar()  Refines return type from object to Foo
 */
class Bar extends Root
{
}

function take_foo(Foo $_f): void
{
}

// Should work - getFoo() is inherited from Root, @method just refines the type
take_foo((new Bar())->getFoo());
take_foo((new Bar())->getBar());

// Test 2: @method with __call present (should work)
/**
 * @method string getQux()
 */
class WithMagicCall
{
    public function __call(string $name, array $arguments): mixed
    {
        return '';
    }
}

(new WithMagicCall())->getQux(); // Should work

// Test 4: Multiple inheritance levels
class GrandParent
{
    /** @return object */
    public function inherited()
    {
        return new Foo();
    }
}

class MiddleClass extends GrandParent
{
}

/** @method Foo inherited() */
class Child extends MiddleClass
{
}

take_foo((new Child())->inherited()); // Should work

// Test 5: @method refining inherited method with different visibility
class BaseClass
{
    /** @return object */
    protected function protectedMethod()
    {
        return new Foo();
    }
}

/**
 * @method Foo protectedMethod()
 */
class DerivedClass extends BaseClass
{
    public function test(): void
    {
        take_foo($this->protectedMethod());
    }
}

(new DerivedClass())->test();
