<?php

// Test 1: Two abstract traits with incompatible parameter types (ERROR)
// PHP: "Declaration of A::process(int $x): string must be compatible with B::process(string $x): string"
trait A
{
    abstract public function process(int $x): string;
}

trait B
{
    abstract public function process(string $x): string;
}

// @mago-expect analysis:unimplemented-abstract-method
class Conflict1
{
    // @mago-expect analysis:incompatible-parameter-type
    use A, B;
}

// Test 2: Two abstract traits with incompatible return types (ERROR)
// PHP: "Declaration of C::getValue(): int must be compatible with D::getValue(): string"
trait C
{
    abstract public function getValue(): int;
}

trait D
{
    abstract public function getValue(): string;
}

// @mago-expect analysis:unimplemented-abstract-method
class Conflict2
{
    use C, D; // @mago-expect analysis:incompatible-return-type
}

// Test 3: Concrete trait method vs abstract trait method with incompatible signature (ERROR)
// PHP: "Declaration of ConcreteT::foo() must be compatible with AbstractT::foo($a)"
trait AbstractT
{
    abstract public function foo($a);
}

trait ConcreteT
{
    public function foo()
    {
        return 42;
    }
}

// @mago-expect analysis:incompatible-parameter-count
class Conflict3
{
    use ConcreteT;
    use AbstractT;
}

// Test 4: Trait conflict with resolution using insteadof (OK if resolved)
// PHP: No error if conflict is resolved
trait E
{
    public function conflict()
    {
        return 'E';
    }
}

trait F
{
    public function conflict()
    {
        return 'F';
    }
}

class Resolved
{
    use E, F {
        E::conflict insteadof F;
    }
}

// Test 5: Multiple traits, one concrete conflicts with abstract (ERROR)
// PHP: Error about incompatibility
trait G
{
    public function method(int $x): void
    {
    }
}

trait H
{
    abstract public function method(string $x): void;
}

// @mago-expect analysis:incompatible-parameter-type
class Conflict4
{
    use G;
    use H;
}

// Test 6: Three traits with conflicting signatures (ERROR)
// PHP: "Declaration of I::multi(int $a) must be compatible with J::multi(string $a)"
// Note: PHP stops at first error, but analyzer may report just one conflict
trait I
{
    abstract public function multi(int $a);
}

trait J
{
    abstract public function multi(string $a);
}

trait K
{
    abstract public function multi(float $a);
}

// @mago-expect analysis:unimplemented-abstract-method
// @mago-expect analysis:incompatible-parameter-type
class Conflict5
{
    use I, J, K;
}

// Test 7: Same trait used multiple times requires implementation (ERROR)
// PHP: "Class NoConflict contains 1 abstract method and must therefore be declared abstract or implement the remaining methods (L::single)"
trait L
{
    abstract public function single();
}

// @mago-expect analysis:unimplemented-abstract-method
class NoConflict
{
    use L;
    use L; // Second use is ignored, but abstract method still needs implementation
}

// Test 8: Trait hierarchy - child trait provides concrete implementation (OK)
// PHP: No error
trait BaseT
{
    abstract public function base(): int;
}

trait ChildT
{
    use BaseT;

    public function base(): int
    {
        return 1;
    }
}

class UsingChildTrait
{
    use ChildT; // OK: Conflict resolved in trait
}

// Test 9: Abstract trait methods with different visibility (ERROR - needs implementation)
// PHP: "Class Conflict6 contains 1 abstract method and must therefore be declared abstract or implement the remaining methods (M::vis)"
// NOTE: Analyzer also reports incompatible-method-signature for visibility mismatch (stricter than PHP)
trait M
{
    abstract public function vis();
}

trait N
{
    abstract protected function vis(); // Different visibility
}

// @mago-expect analysis:unimplemented-abstract-method
// @mago-expect analysis:incompatible-visibility
class Conflict6
{
    use M, N;
}

// Test 10: Both traits have same compatible abstract signature (ERROR - needs implementation)
// PHP: "Class NoConflict2 contains 1 abstract method and must therefore be declared abstract or implement the remaining methods (O::same)"
trait O
{
    abstract public function same(string $x): int;
}

trait P
{
    abstract public function same(string $x): int;
}

// @mago-expect analysis:unimplemented-abstract-method
class NoConflict2
{
    use O, P; // Same signature from both traits, but still needs implementation
}
