<?php

// Test 1: Different typed properties (ERROR)
// PHP: "T1 and T2 define the same property ($prop) in the composition of C1. However, the definition differs and is considered incompatible."
trait T1
{
    public string $prop;
}

trait T2
{
    public int $prop;
}

// @mago-expect analysis:missing-constructor
class C1
{
    // @mago-expect analysis:incompatible-property-type
    use T1, T2;
}

// Test 2: Typed vs untyped property (ERROR)
// PHP: Fatal error
trait T3
{
    public string $prop;
}

trait T4
{
    public $prop;
}

// @mago-expect analysis:missing-constructor
class C2
{
    // @mago-expect analysis:incompatible-property-type
    use T3, T4;
}

// Test 3: Same type (OK)
// PHP: No error
trait T5
{
    public string $prop;
}

trait T6
{
    public string $prop;
}

// @mago-expect analysis:missing-constructor
class C3
{
    use T5, T6; // OK: Both string
}

// Test 4: Nullable vs non-nullable (ERROR)
// PHP: Fatal error
trait T7
{
    public null|string $prop;
}

trait T8
{
    public string $prop;
}

// @mago-expect analysis:missing-constructor
class C4
{
    // @mago-expect analysis:incompatible-property-type
    use T7, T8;
}

// Test 5: Union types - different (ERROR)
// PHP: Fatal error (PHP 8.0+)
trait T9
{
    public string|int $prop;
}

trait T10
{
    public string|float $prop;
}

// @mago-expect analysis:missing-constructor
class C5
{
    // @mago-expect analysis:incompatible-property-type
    use T9, T10;
}

// Test 6: Same union type (OK)
// PHP: No error
trait T11
{
    public string|int $prop;
}

trait T12
{
    public string|int $prop;
}

// @mago-expect analysis:missing-constructor
class C6
{
    use T11, T12; // OK: Same union
}

// Test 7: Union types with different order (OK - PHP normalizes them)
// PHP: No error (order doesn't matter for union types)
trait T13
{
    public string|int $prop;
}

trait T14
{
    public int|string $prop; // Different order, but equivalent
}

// @mago-expect analysis:missing-constructor
class C7
{
    use T13, T14; // OK: Both are equivalent
}

// Test 8: Array types (ERROR)
// PHP: Fatal error
trait T15
{
    public array $prop;
}

trait T16
{
    public $prop; // Untyped
}

// @mago-expect analysis:missing-constructor
class C8
{
    // @mago-expect analysis:incompatible-property-type
    use T15, T16;
}

// Test 9: Class types (ERROR)
// PHP: Fatal error
trait T17
{
    public stdClass $prop;
}

trait T18
{
    public Exception $prop;
}

// @mago-expect analysis:missing-constructor
class C9
{
    // @mago-expect analysis:incompatible-property-type
    use T17, T18;
}

// Test 10: Same class type (OK)
// PHP: No error
trait T19
{
    public DateTime $prop;
}

trait T20
{
    public DateTime $prop;
}

// @mago-expect analysis:missing-constructor
class C10
{
    use T19, T20; // OK: Same class
}

// Test 11: Intersection types (ERROR)
// PHP: Fatal error (PHP 8.1+)
trait T21
{
    public Countable&Iterator $prop;
}

trait T22
{
    public Countable&Traversable $prop;
}

// @mago-expect analysis:missing-constructor
class C11
{
    // @mago-expect analysis:incompatible-property-type
    use T21, T22;
}
