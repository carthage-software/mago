<?php

// Test 1: Multiple simultaneous violations
// PHP: Reports first/most critical error
interface I1
{
    public function multi(string $a, int $b): bool;
}

class C1 implements I1
{
    // Multiple issues: narrowed param type, extra required param, different return type
    // First error reported: visibility narrowed from public to protected
    // @mago-expect analysis:incompatible-visibility
    protected function multi(User $a, int $b, array $c): int
    {
        return 0;
    }
}

// Test 2: Interface + trait requirements combined
// PHP: Must satisfy both
interface I2
{
    public function combined(string $x): int;
}

trait T1
{
    abstract public function combined(string $x): int;
}

class C2 implements I2
{
    use T1;

    // OK: Satisfies both interface and trait
    public function combined(string $x): int
    {
        return 1;
    }
}

// Test 3: Interface + trait with conflicting requirements
// PHP: No error - mixed type is compatible with both int and string
// Analyzer: Reports incompatible-method-signature (stricter than PHP)
interface I3
{
    public function conflict(int $x): string;
}

trait T2
{
    abstract public function conflict(string $x): string;
}

class C3 implements I3
{
    use T2;

    // @mago-expect analysis:incompatible-parameter-type
    // PHP accepts mixed as satisfying both, but analyzer is stricter
    public function conflict(mixed $x): string
    {
        return '';
    }
}

// Test 4: Inheritance chain with multiple abstract methods
// PHP: All must be compatible
abstract class Base1
{
    abstract public function chain(object $x): void;
}

abstract class Base2 extends Base1
{
    // Narrows parameter type
    // @mago-expect analysis:incompatible-parameter-type
    abstract public function chain(User $x): void;
}

class Concrete extends Base2
{
    public function chain(User $x): void
    {
    }
}

// Test 5: Diamond problem with interfaces
// PHP: Must be compatible with all
interface IA
{
    public function diamond(string $x);
}

interface IB extends IA
{
    public function diamond(string $x);
}

interface IC extends IA
{
    public function diamond(string $x);
}

class Diamond implements IB, IC
{
    // OK: All require same signature
    public function diamond(string $x)
    {
    }
}

// Test 6: Mixed static and instance methods
// PHP: Each checked independently
interface I4
{
    public static function staticMethod(): void;

    public function instanceMethod(): void;
}

class C4 implements I4
{
    // @mago-expect analysis:incompatible-static-modifier
    public function staticMethod(): void
    {
    } // ERROR: Should be static

    public function instanceMethod(): void
    {
    } // OK
}

// Test 7: Optional parameters with type changes
// PHP: Type must still be compatible
interface I5
{
    public function optional(string $x = 'default'): void;
}

class C5 implements I5
{
    // ERROR: Parameter type incompatible even though optional
    // @mago-expect analysis:incompatible-parameter-type
    public function optional(int $x = 0): void
    {
    }
}

// Test 8: Variadic parameters
// PHP: Variadic must be compatible
interface I6
{
    public function variadic(string ...$args): void;
}

class C6 implements I6
{
    // OK: Same variadic signature
    public function variadic(string ...$args): void
    {
    }
}

class C7 implements I6
{
    // ERROR: Different type for variadic
    // @mago-expect analysis:incompatible-parameter-type
    public function variadic(int ...$args): void
    {
    }
}

// Test 9: By-reference parameters
// PHP: Reference status must match
interface I7
{
    public function byRef(array &$data): void;
}

class C8 implements I7
{
    // OK: Both by-reference
    public function byRef(array &$data): void
    {
    }
}

// Test 10: Method exists in class before trait use
// PHP: Class method takes precedence but may conflict
trait T3
{
    abstract public function preexisting(int $x): void;
}

// @mago-expect analysis:incompatible-parameter-type
class C9
{
    public function preexisting(string $x): void
    {
    } // Defined first

    use T3;
}

class User
{
}
