<?php

// Test 1: Trait provides concrete implementation of another trait's abstract method (OK)
// PHP: No error - conflict resolved in trait hierarchy
trait AbstractTrait
{
    abstract public function foo(): int;
}

trait ConcreteTrait
{
    use AbstractTrait;

    public function foo(): int
    {
        return 42;
    }
}

class User1
{
    use ConcreteTrait; // OK: Abstract method satisfied by trait
}

// Test 2: Class method defined before trait use with incompatible signature (ERROR)
// PHP: "T2::bar() and C2::bar() define the same method (bar) in the composition of C2"
trait T2
{
    abstract public function bar(string $x): void;
}

class C2
{
    use T2;

    // @mago-expect analysis:incompatible-parameter-type
    public function bar(int $x): void
    {
    }
}

// Test 3: Private abstract method in trait
// PHP: "Class AbstractClass contains 1 abstract method and must therefore be declared abstract or implement the remaining methods (T3::privateMethod)"
// Note: PHP requires even abstract classes to implement private abstract methods from traits
// This is a unique PHP behavior that may not be enforced by all analyzers
trait T3
{
    abstract private function privateMethod(): void;
}

abstract class AbstractClass
{
    use T3; // PHP ERROR (not currently caught by analyzer): Even abstract class must implement private abstract from trait
}

// Note: Concrete class would need to implement even private abstract from trait
class C3
{
    use T3;

    // Must implement even though private
    private function privateMethod(): void
    {
    }
}

// Test 4: Trait with abstract static method
// PHP: Works same as regular abstract
trait T4
{
    abstract public static function factory(): self;
}

class C4
{
    use T4;

    public static function factory(): self
    {
        return new self();
    }
}

// Test 5: Interface method implemented by trait
// PHP: OK if trait provides implementation
interface I1
{
    public function required(): string;
}

trait T5
{
    public function required(): string
    {
        return 'implemented';
    }
}

class C5 implements I1
{
    use T5; // OK: Trait implements interface method
}

// Test 6: Multiple inheritance levels with method override
// PHP: Each level must be compatible with parent
interface Base
{
    public function process(mixed $x): void;
}

interface Middle extends Base
{
    public function process(mixed $x): void;
}

interface Leaf extends Middle
{
    public function process(mixed $x): void;
}

class Implementation implements Leaf
{
    public function process(mixed $x): void
    {
    }
}

// Test 7: Self return type in trait abstract method
// PHP: self refers to class using trait
trait T6
{
    abstract public function builder(): self;
}

class C6
{
    use T6;

    public function builder(): self
    {
        return $this;
    }
}

// Test 8: Trait aliasing with visibility change
// PHP: Can change visibility with 'as'
trait T7
{
    public function publicMethod()
    {
    }
}

class C7
{
    use T7 {
        publicMethod as protected;
    }
}

// Test 9: Abstract method in anonymous class
// PHP: Anonymous classes can be abstract
$abstract = new class {
    use AbstractTrait; // Has abstract method

    public function foo(): int
    {
        return 1;
    }
};

// Test 10: Trait method conflict resolved with alias
// PHP: OK when aliased
trait T8
{
    public function conflict()
    {
        return 'T8';
    }
}

trait T9
{
    public function conflict()
    {
        return 'T9';
    }
}

class C8
{
    use T8, T9 {
        T8::conflict as conflictFromT8;
        T9::conflict as conflictFromT9;
    }

    // Must still resolve original conflict
    public function conflict()
    {
        return 'resolved';
    }
}
