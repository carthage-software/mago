<?php

// Test 1: Basic trait constant override - NOT allowed in class that uses trait
trait FooTrait
{
    public const TEST = 1;
}

class Foo
{
    use FooTrait;

    public const TEST = 2; // @mago-expect analysis:trait-constant-override
}

// Test 2: Override in child class - ALLOWED (inheriting from parent, not directly from trait)
class Bar extends Foo
{
    public const TEST = 3; // ok - overriding parent class constant
}

// Test 3: Multiple traits with same constant - conflict resolution then override
trait TraitA
{
    public const SHARED = 10;
}

trait TraitB
{
    public const SHARED = 20;
}

class MultiTraitUser
{
    use TraitA, TraitB {
        TraitA::SHARED insteadof TraitB;
    }

    // Even with conflict resolution, can't override in same class
    public const SHARED = 30; // @mago-expect analysis:trait-constant-override
}

// Test 4: Trait hierarchy - constant from parent trait
trait ParentTrait
{
    public const VALUE = 'parent';
}

trait ChildTrait
{
    use ParentTrait;
}

class UsingChildTrait
{
    use ChildTrait;

    // Can't override constant that came from trait (even indirectly)
    public const VALUE = 'override'; // @mago-expect analysis:trait-constant-override
}

// Test 5: Grandchild can override
class GrandChild extends UsingChildTrait
{
    public const VALUE = 'grandchild'; // ok - overriding inherited constant
}

// Test 6: Class without trait usage can define same constant
class Independent
{
    public const TEST = 999; // ok - not using FooTrait
}

// Test 7: Same value - ALLOWED (PHP allows redeclaring with same value)
trait SameValueTrait
{
    public const SAME = 42;
}

class SameValueClass
{
    use SameValueTrait;

    public const SAME = 42; // ok - same value as trait
}

// Test 8: No trait, parent class inheritance works fine
class ParentClass
{
    public const PARENT_CONST = 'parent';
}

class ChildClass extends ParentClass
{
    public const PARENT_CONST = 'child'; // ok - normal inheritance override
}
