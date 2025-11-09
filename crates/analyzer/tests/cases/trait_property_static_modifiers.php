<?php

// Test 1: Static vs instance property (ERROR)
// PHP: "T1 and T2 define the same property ($prop) in the composition of C1. However, the definition differs and is considered incompatible."
trait T1
{
    public static $prop;
}

trait T2
{
    public $prop;
}

class C1
{
    // @mago-expect analysis:incompatible-property-static
    use T1, T2;
}

// Test 2: Both static - different visibility (ERROR)
// PHP: Fatal error
trait T3
{
    public static $prop;
}

trait T4
{
    private static $prop;
}

class C2
{
    // @mago-expect analysis:incompatible-property-visibility
    use T3, T4;
}

// Test 3: Both static - same visibility (OK)
// PHP: No error
trait T5
{
    public static $prop;
}

trait T6
{
    public static $prop;
}

class C3
{
    use T5, T6; // OK: Both public static
}

// Test 4: Static with different defaults (ERROR)
// PHP: Fatal error
trait T7
{
    public static $prop = 1;
}

trait T8
{
    public static $prop = 2;
}

class C4
{
    // @mago-expect analysis:incompatible-property-default
    use T7, T8;
}

// Test 5: Static with same defaults (OK)
// PHP: No error
trait T9
{
    public static $prop = 'value';
}

trait T10
{
    public static $prop = 'value';
}

class C5
{
    use T9, T10; // OK: Identical
}

// Test 6: Three traits - static conflicts (ERROR)
// PHP: Fatal error
trait T11
{
    public static $prop;
}

trait T12
{
    protected static $prop;
}

trait T13
{
    public $prop; // Instance
}

class C6
{
    // @mago-expect analysis:incompatible-property-visibility
    // @mago-expect analysis:incompatible-property-visibility
    // @mago-expect analysis:incompatible-property-static
    use T11, T12, T13;
}

// Test 7: Static property access pattern (OK)
// PHP: No error
trait T14
{
    public static int $counter = 0;

    public function increment()
    {
        self::$counter++;
    }
}

trait T15
{
    public static int $counter = 0;
}

class C7
{
    use T14, T15; // OK: Both identical
}
