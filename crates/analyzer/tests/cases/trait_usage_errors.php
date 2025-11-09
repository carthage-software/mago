<?php

// Test 1: Using undefined trait
class UsingUndefinedTrait
{
    use NonExistentTrait; // @mago-expect analysis:non-existent-class-like
}

// Test 2: Using interface as trait
interface SomeInterface
{
    public function foo();
}

class UsingInterfaceAsTrait // @mago-expect analysis:unimplemented-abstract-method
{
    use SomeInterface; // @mago-expect analysis:invalid-trait-use
}

// Test 3: Using class as trait
class SomeClass
{
    public function bar()
    {
    }
}

class UsingClassAsTrait
{
    use SomeClass; // @mago-expect analysis:invalid-trait-use
}

// Test 4: Using final class as trait
final class FinalClass
{
    public function baz()
    {
    }
}

class UsingFinalClassAsTrait
{
    use FinalClass; // @mago-expect analysis:invalid-trait-use
}

// Test 5: Using abstract class as trait
abstract class AbstractClass
{
    abstract public function qux();
}

class UsingAbstractClassAsTrait // @mago-expect analysis:unimplemented-abstract-method
{
    use AbstractClass; // @mago-expect analysis:invalid-trait-use
}

// Test 6: Instantiating a trait
trait InstantiableTrait
{
    public function method()
    {
    }
}

new InstantiableTrait(); // @mago-expect analysis:trait-instantiation

// Test 7: Implementing a trait (as if it were an interface)
trait ImplementableTrait
{
    public function method()
    {
    }
}

class ImplementingTrait implements ImplementableTrait // @mago-expect analysis:invalid-implement
{
}

// Test 8: Extending a trait (as if it were a class)
trait ExtendableTrait
{
    public function method()
    {
    }
}

class ExtendingTrait extends ExtendableTrait // @mago-expect analysis:invalid-extend
{
}

// Valid cases (should not error)
trait ValidTrait
{
    public function validMethod()
    {
    }
}

class ValidUsage
{
    use ValidTrait; // OK
}
