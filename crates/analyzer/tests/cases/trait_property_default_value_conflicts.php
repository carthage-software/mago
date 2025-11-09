<?php

// Test 1: Different default values (ERROR)
// PHP: "T1 and T2 define the same property ($prop) in the composition of C1. However, the definition differs and is considered incompatible."
trait T1
{
    public $prop = 1;
}

trait T2
{
    public $prop = 2;
}

class C1
{
    // @mago-expect analysis:incompatible-property-default
    use T1, T2;
}

// Test 2: Default value vs no default value (ERROR)
// PHP: Same fatal error
trait T3
{
    public $prop = 'hello';
}

trait T4
{
    public $prop;
}

class C2
{
    // @mago-expect analysis:incompatible-property-default
    use T3, T4;
}

// Test 3: Same default value (OK)
// PHP: No error
trait T5
{
    public $prop = 42;
}

trait T6
{
    public $prop = 42;
}

class C3
{
    use T5, T6; // OK: Both have same default value
}

// Test 4: Different array default values (ERROR)
// PHP: Fatal error
trait T7
{
    public $prop = [1, 2, 3];
}

trait T8
{
    public $prop = [1, 2, 4];
}

class C4
{
    // @mago-expect analysis:incompatible-property-default
    use T7, T8;
}

// Test 5: String vs integer default (ERROR)
// PHP: Fatal error
trait T9
{
    public $prop = '42';
}

trait T10
{
    public $prop = 42;
}

class C5
{
    // @mago-expect analysis:incompatible-property-default
    use T9, T10;
}

// Test 6: Null vs no default (ERROR)
// PHP: Fatal error
trait T11
{
    public $prop = null;
}

trait T12
{
    public $prop;
}

class C6
{
    // @mago-expect analysis:incompatible-property-default
    use T11, T12;
}

// Test 7: Complex object default values (ERROR)
// PHP: Fatal error
trait T13
{
    public $prop = ['key' => 'value1'];
}

trait T14
{
    public $prop = ['key' => 'value2'];
}

class C7
{
    // @mago-expect analysis:incompatible-property-default
    use T13, T14;
}

// Test 8: Same complex default (OK)
// PHP: No error
trait T15
{
    public $prop = ['x' => 1, 'y' => 2];
}

trait T16
{
    public $prop = ['x' => 1, 'y' => 2];
}

class C8
{
    use T15, T16; // OK: Both have identical array
}

// Test 9: Three traits with different defaults (ERROR)
// PHP: Fatal error
trait T17
{
    public $prop = 'a';
}

trait T18
{
    public $prop = 'b';
}

trait T19
{
    public $prop = 'c';
}

class C9
{
    // @mago-expect analysis:incompatible-property-default
    use T17, T18, T19;
}
