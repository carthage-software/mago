<?php

// Test 1: Public vs Private conflict (ERROR)
// PHP: "T1 and T2 define the same property ($prop) in the composition of C1. However, the definition differs and is considered incompatible."
trait T1
{
    public $prop;
}

trait T2
{
    private $prop;
}

class C1
{
    // @mago-expect analysis:incompatible-property-visibility
    use T1, T2;
}

// Test 2: Public vs Protected conflict (ERROR)
// PHP: Same fatal error message
trait T3
{
    public $prop;
}

trait T4
{
    protected $prop;
}

class C2
{
    // @mago-expect analysis:incompatible-property-visibility
    use T3, T4;
}

// Test 3: Protected vs Private conflict (ERROR)
// PHP: Same fatal error message
trait T5
{
    protected $prop;
}

trait T6
{
    private $prop;
}

class C3
{
    // @mago-expect analysis:incompatible-property-visibility
    use T5, T6;
}

// Test 4: Same visibility - no conflict (OK)
// PHP: No error
trait T7
{
    public $prop;
}

trait T8
{
    public $prop;
}

class C4
{
    use T7, T8; // OK: Both public, both uninitialized
}

// Test 5: Asymmetric visibility conflict (PHP 8.4+)
// PHP: Fatal error
trait T9
{
    public private(set) $prop;
}

trait T10
{
    public protected(set) $prop;
}

class C5
{
    // @mago-expect analysis:incompatible-property-visibility
    use T9, T10;
}

// Test 6: Three traits with visibility conflicts (ERROR)
// PHP: Fatal error
trait T11
{
    public $prop;
}

trait T12
{
    protected $prop;
}

trait T13
{
    private $prop;
}

class C6
{
    // @mago-expect analysis:incompatible-property-visibility
    use T11, T12, T13;
}

// Test 7: Trait hierarchy visibility conflict (ERROR)
// PHP: Fatal error
trait Base
{
    private $prop;
}

trait Child
{
    use Base;
    // @mago-expect analysis:incompatible-property-visibility
    public $prop; // Conflicts with inherited private $prop
}

class C7
{
    use Child;
}

// Test 8: Multiple use statements with conflict (ERROR)
// PHP: Fatal error
trait T14
{
    public $prop;
}

trait T15
{
    private $prop;
}

class C8
{
    use T14;
    // @mago-expect analysis:incompatible-property-visibility
    use T15; // Second use statement introduces conflict
}
