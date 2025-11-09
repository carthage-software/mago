<?php

// Test 1: Trait using trait with property conflict (ERROR)
// PHP: "T1 and T2 define the same property ($prop) in the composition of T3."
trait T1
{
    public $prop = 1;
}

trait T2
{
    public $prop = 2;
}

trait T3
{
    // @mago-expect analysis:incompatible-property-default
    use T1, T2;
}

class C1
{
    use T3;
}

// Test 2: Multi-level trait hierarchy (ERROR)
// PHP: Fatal error
trait Base1
{
    public $prop = 'base1';
}

trait Base2
{
    public $prop = 'base2';
}

trait Middle
{
    use Base1;
}

trait Another
{
    use Base2;
}

class C2
{
    // @mago-expect analysis:incompatible-property-default
    use Middle, Another;
}

// Test 3: Trait overriding inherited trait property (ERROR)
// PHP: Fatal error
trait Parent
{
    public $prop = 1;
}

trait Child
{
    use Parent;
    // @mago-expect analysis:incompatible-property-default
    public $prop = 2; // Conflicts with inherited property
}

class C3
{
    use Child;
}

// Test 4: Diamond problem with properties (ERROR)
// PHP: Fatal error
trait A
{
    public $prop = 'A';
}

trait B
{
    use A;
}

trait C
{
    use A;
    // @mago-expect analysis:incompatible-property-default
    public $prop = 'C'; // Redefines from A
}

trait D
{
    // @mago-expect analysis:incompatible-property-default
    use B, C;
}

class C4
{
    use D;
}

// Test 5: Resolved hierarchy (OK)
// PHP: No error
trait R1
{
    public $prop = 42;
}

trait R2
{
    use R1;
}

trait R3
{
    use R1;
}

class C5
{
    use R2, R3; // OK: Both resolve to same property from R1
}

// Test 6: Complex hierarchy with visibility (ERROR)
// PHP: Fatal error
trait V1
{
    public $prop;
}

trait V2
{
    use V1;
    // @mago-expect analysis:incompatible-property-visibility
    protected $prop; // Different visibility
}

class C6
{
    use V2;
}

// Test 7: Multiple trait uses at different levels (ERROR)
// PHP: Fatal error
trait Level0
{
    public $prop = 0;
}

trait Level1A
{
    use Level0;
    // @mago-expect analysis:incompatible-property-default
    public $prop = 1;
}

trait Level1B
{
    use Level0;
}

class C7
{
    // @mago-expect analysis:incompatible-property-default
    use Level1A, Level1B;
}

// Test 8: Three-level hierarchy (ERROR)
// PHP: Fatal error
trait T1Deep
{
    public $prop = 'deep';
}

trait T2Deep
{
    use T1Deep;
}

trait T3Deep
{
    use T2Deep;
    // @mago-expect analysis:incompatible-property-default
    public $prop = 'override'; // Conflicts with inherited
}

class C8
{
    use T3Deep;
}
