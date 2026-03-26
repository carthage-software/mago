<?php

class A
{
    /**
     * @mago-expect analysis:write-only-property
     * @var int
     */
    private $foo;

    public function __construct(int &$foo)
    {
        $this->foo = &$foo;
    }
}

class B
{
    /**
     * @mago-expect analysis:write-only-property
     * @var string
     */
    private $bar;

    public function __construct(string &$bar)
    {
        $this->bar = &$bar;
    }
}

function get_bool(): bool
{
    return get_bool();
}

/**
 * @mago-expect analysis:conflicting-reference-constraint
 * @mago-expect analysis:reference-constraint-violation
 */
function constraint_conflict(): void
{
    if (get_bool()) {
        $v = 5;
        $c = new A($v); // $v is constrained to an int
    } else {
        $v = 'hello';
        $c = new B($v); // $v is constrained to a string
    }

    $v = []; // constraint violation (`int` <- `array`)
}

/**
 * @mago-expect analysis:conflicting-reference-constraint
 * @mago-expect analysis:reference-constraint-violation
 */
function constraint_conflict_in_switch(): void
{
    switch (get_bool()) {
        case true:
            $v = 5;
            $c = new A($v); // $v is constrained to an int
            break;
        default:
            $v = 'hello';
            $c = new B($v); // $v is constrained to a string
            break;
    }

    $v = []; // constraint violation (`int` <- `array`)
}

/**
 * @mago-expect analysis:conflicting-reference-constraint
 * @mago-expect analysis:reference-constraint-violation
 */
function constraint_conflict_in_try(): void
{
    try {
        $v = 5;
        $c = new A($v); // $v is constrained to an int
    } catch (Throwable $e) {
        $v = 'hello';
        $c = new B($v); // $v is constrained to a string
    }

    $v = []; // constraint violation (`int` <- `array`)
}

/**
 * @mago-expect analysis:conflicting-reference-constraint
 * @mago-expect analysis:reference-constraint-violation
 */
function constraint_conflict_in_loop(): void
{
    $v = 5;
    $c = new A($v); // $v is constrained to an int
    for ($i = 0; get_bool(); $i++) {
        $v = 'hello';
        $c = new B($v); // $v is constrained to a string
    }

    $v = []; // constraint violation (`int` <- `array`)
}

/**
 * @mago-expect analysis:reference-constraint-violation
 */
function arg_constraint_violation(): void
{
    $v = 5;
    $c = new A($v);
    $v = 'hello'; // constraint violation
}

/**
 * @mago-expect analysis:reference-constraint-violation
 */
function param_constraint_violation(string &$str): void
{
    $str = 12; // constraint violation
}

/**
 * @mago-expect analysis:reference-constraint-violation
 */
function &static_constraint_violation(): array
{
    /** @var string $foo */
    static $foo = 'hello';

    $foo = []; // static constraint violation

    return $foo;
}
