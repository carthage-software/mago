<?php

// Test 1: Static to non-static (ERROR)
// PHP: "Cannot make static method I1::create() non-static in class C1"
interface I1
{
    public static function create(): static;
}

/**
 * @psalm-consistent-constructor
 */
class C1 implements I1
{
    // @mago-expect analysis:incompatible-static-modifier
    public function create(): static
    {
        return new static();
    }
}

// Test 2: Non-static to static (ERROR)
// PHP: "Cannot make non-static method Base::instance() static in class Child"
abstract class Base
{
    abstract public function instance(): void;
}

class Child extends Base
{
    // @mago-expect analysis:incompatible-static-modifier
    public static function instance(): void
    {
    }
}

// Test 3: Both static (OK)
// PHP: No error
interface I2
{
    public static function build(): self;
}

class C2 implements I2
{
    // OK: Both static
    public static function build(): self
    {
        return new self();
    }
}

// Test 4: Both non-static (OK)
// PHP: No error
abstract class Base2
{
    abstract public function process(): void;
}

class Child2 extends Base2
{
    // OK: Both non-static
    public function process(): void
    {
    }
}

// Test 5: Trait abstract static, class non-static (ERROR)
// PHP: "Cannot make static method T1::factory() non-static in class C3"
trait T1
{
    abstract public static function factory(): self;
}

class C3
{
    use T1;

    // @mago-expect analysis:incompatible-static-modifier
    public function factory(): self
    {
        return $this;
    }
}

// Test 6: Trait abstract non-static, class static (ERROR)
// PHP: "Cannot make non-static method T2::execute() static in class C4"
trait T2
{
    abstract public function execute(): void;
}

class C4
{
    use T2;

    // @mago-expect analysis:incompatible-static-modifier
    public static function execute(): void
    {
    }
}

// Test 7: Interface static method inheritance (OK)
// PHP: No error
interface Factory
{
    public static function make(string $type): object;
}

interface SpecialFactory extends Factory
{
    // OK: Can redeclare as static
    public static function make(string $type): object;
}
