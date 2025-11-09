<?php

// Test 1: Readonly vs non-readonly (ERROR)
// PHP: "T1 and T2 define the same property ($prop) in the composition of C1. However, the definition differs and is considered incompatible." (PHP 8.1+)
trait T1
{
    public readonly string $prop;
}

trait T2
{
    public string $prop;
}

class C1
{
    // @mago-expect analysis:incompatible-property-visibility
    use T1, T2;
}

// Test 2: Both readonly - different types (ERROR)
// PHP: Fatal error
trait T3
{
    public readonly string $prop;
}

trait T4
{
    public readonly int $prop;
}

class C2
{
    // @mago-expect analysis:incompatible-property-type
    use T3, T4;
}

// Test 3: Both readonly - same type (OK)
// PHP: No error
trait T5
{
    public readonly string $prop;
}

trait T6
{
    public readonly string $prop;
}

class C3
{
    use T5, T6; // OK: Both readonly string
}

// Test 4: Readonly with different visibility (ERROR)
// PHP: Fatal error
trait T7
{
    public readonly string $prop;
}

trait T8
{
    protected readonly string $prop;
}

class C4
{
    // @mago-expect analysis:incompatible-property-visibility
    use T7, T8;
}

// Test 5: Readonly class using readonly trait (OK)
// PHP: No error
trait T9
{
    public readonly string $name;
    public readonly int $age;
}

readonly class C5
{
    use T9; // OK: Class is readonly
}

// Test 6: Non-readonly class with readonly trait property (OK)
// PHP: No error - readonly properties are allowed in non-readonly classes
trait T10
{
    public readonly string $prop;
}

class C6
{
    use T10; // OK
}

// Test 7: Readonly property without type (ERROR in PHP)
// PHP: "Readonly property must have type"
// Note: This is a different error, not a conflict
trait T11
{
    public readonly $prop; // Invalid in PHP 8.1+
}

// Test 8: Multiple readonly properties (OK)
// PHP: No error
trait T12
{
    public readonly string $prop1;
    public readonly int $prop2;
}

trait T13
{
    public readonly string $prop1;
    public readonly int $prop2;
}

class C7
{
    use T12, T13; // OK: All properties match
}

// Test 9: Readonly static property (ERROR in PHP)
// PHP: "Readonly property cannot be static"
// Note: This is a different validation error
trait T14
{
    public static readonly string $prop; // Invalid
}

// Test 10: Readonly conflict in hierarchy (ERROR)
// PHP: Fatal error
trait Base
{
    public readonly string $prop;
}

trait Child
{
    use Base;
    // @mago-expect analysis:incompatible-property-visibility
    public string $prop; // Not readonly - conflicts
}

class C8
{
    use Child;
}
