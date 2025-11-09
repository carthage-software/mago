<?php

// Test 1: Public to protected narrowing in interface (ERROR)
// PHP: "Access level to C1::foo() must be public (as in class I1)"
interface I1
{
    public function foo();
}

class C1 implements I1
{
    // @mago-expect analysis:incompatible-visibility
    protected function foo()
    {
    }
}

// Test 2: Public to private narrowing in interface (ERROR)
// PHP: "Access level to C2::bar() must be public (as in class I2)"
interface I2
{
    public function bar();
}

class C2 implements I2
{
    // @mago-expect analysis:incompatible-visibility
    private function bar()
    {
    }
}

// Test 3: Protected to private narrowing in abstract class (ERROR)
// PHP: "Access level to Child::baz() must be protected (as in class Base) or weaker"
abstract class Base
{
    abstract protected function baz();
}

class Child extends Base
{
    // @mago-expect analysis:incompatible-visibility
    private function baz()
    {
    }
}

// Test 4: Protected to public widening (OK)
// PHP: No error
abstract class Base2
{
    abstract protected function process();
}

class Child2 extends Base2
{
    // OK: Widening visibility
    public function process()
    {
    }
}

// Note: Tests 5-6 removed - PHP does not support abstract private methods (parse error)
// This is invalid PHP syntax: "Abstract function cannot be declared private"

// Test 7: Same visibility level (OK)
// PHP: No error
interface I3
{
    public function execute();
}

class C3 implements I3
{
    // OK: Same visibility
    public function execute()
    {
    }
}

// Test 8: Trait abstract method visibility narrowing (ERROR)
// PHP: "Access level to C4::save() must be public (as in class I4)"
// Note: Traits don't enforce visibility on their own, but interfaces do
interface I4
{
    public function save();
}

trait T1
{
    abstract public function save();
}

class C4 implements I4
{
    use T1;

    // @mago-expect analysis:incompatible-visibility
    protected function save()
    {
    }
}
