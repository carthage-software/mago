<?php

// Test 1: Class property with different default than trait (ERROR)
// PHP: "C1 and T1 define the same property ($prop) in the composition of C1. However, the definition differs and is considered incompatible."
trait T1
{
    public $prop = 1;
}

class C1
{
    // @mago-expect analysis:incompatible-property-default
    public $prop = 2;
    use T1;
}

// Test 2: Class property with different visibility (ERROR)
// PHP: Fatal error
trait T2
{
    public $prop;
}

class C2
{
    // @mago-expect analysis:incompatible-property-visibility
    // @mago-expect analysis:unused-property
    private $prop;
    use T2;
}

// Test 3: Class property before and after trait use (ERROR)
// PHP: Fatal error
trait T3
{
    public $prop = 'trait';
}

class C3
{
    use T3;
    // @mago-expect analysis:incompatible-property-default
    public $prop = 'class'; // Position doesn't matter
}

// Test 4: Identical class and trait property (OK)
// PHP: No error
trait T4
{
    public $prop = 42;
}

class C4
{
    public $prop = 42;
    use T4; // OK: Identical definition
}

// Test 5: Class static vs trait instance (ERROR)
// PHP: Fatal error
trait T5
{
    public $prop;
}

class C5
{
    // @mago-expect analysis:incompatible-property-static
    public static $prop;
    use T5;
}

// Test 6: Class typed vs trait untyped (ERROR)
// PHP: Fatal error
trait T6
{
    public $prop;
}

class C6
{
    // @mago-expect analysis:incompatible-property-type
    public string $prop = '';
    use T6;
}

// Test 7: Multiple traits and class property (ERROR)
// PHP: Fatal error - reports first conflict found
trait T7
{
    public $prop = 1;
}

trait T8
{
    public $prop = 2;
}

class C7
{
    // @mago-expect analysis:incompatible-property-default
    public $prop = 3;
    // @mago-expect analysis:incompatible-property-default
    use T7, T8; // Also conflicts between T7 and T8
}

// Test 8: Class readonly vs trait non-readonly (ERROR)
// PHP: Fatal error (PHP 8.1+)
trait T9
{
    public $prop;
}

class C8
{
    // @mago-expect analysis:incompatible-property-visibility
    public readonly string $prop = '';
    use T9;
}
