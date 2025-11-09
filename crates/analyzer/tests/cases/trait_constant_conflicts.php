<?php

// Test 1: Visibility conflict (public trait -> private class)
trait PublicConstantTrait
{
    public const VALUE = 1;
}

class PrivateConstantClass
{
    use PublicConstantTrait;

    // @mago-expect analysis:incompatible-constant-override
    private const VALUE = 1;
}

// Test 2: Visibility conflict (protected trait -> private class)
trait ProtectedConstantTrait
{
    protected const CODE = 42;
}

class PrivateCodeClass
{
    use ProtectedConstantTrait;

    // @mago-expect analysis:incompatible-constant-override
    private const CODE = 42;
}

// Test 3: Finality conflict (non-final trait -> final class)
trait RegularTrait
{
    public const STATUS = 'active';
}

class FinalConstantClass
{
    use RegularTrait;

    // @mago-expect analysis:incompatible-constant-override
    final public const STATUS = 'active';
}

// Test 4: Nested trait conflict (trait using trait)
trait BaseTrait
{
    public const ID = 100;
}

trait NestedTrait
{
    use BaseTrait;

    // @mago-expect analysis:incompatible-constant-override
    private const ID = 100;
}

// Test 5: Multiple conflicts (visibility AND finality)
trait PublicFinalTrait
{
    final public const NAME = 'test';
}

class PrivateNonFinalClass
{
    use PublicFinalTrait;

    // @mago-expect analysis:incompatible-constant-override
    private const NAME = 'test';
}

// Test 6: Value conflict (uses trait-constant-override for value-only changes)
trait ValueTrait
{
    public const NUMBER = 10;
}

class DifferentValueClass
{
    use ValueTrait;

    // @mago-expect analysis:trait-constant-override
    public const NUMBER = 20;
}

// Test 7: Valid - exact match
trait ValidTrait
{
    public const VALID = true;
}

class ValidClass
{
    use ValidTrait;

    public const VALID = true; // No error - exact match
}

// Test 8: Even widening visibility is NOT allowed for trait constants
trait ProtectedWidenTrait
{
    protected const WIDEN = 'yes';
}

class PublicWidenClass
{
    use ProtectedWidenTrait;

    // @mago-expect analysis:incompatible-constant-override
    public const WIDEN = 'yes'; // ERROR - even widening is not allowed
}

//Test 9: Nested trait with multiple levels
trait GrandBaseTrait
{
    public const LEVEL = 1;
}

trait ParentTrait
{
    use GrandBaseTrait;
}

trait ChildTrait
{
    use ParentTrait;

    // @mago-expect analysis:incompatible-constant-override
    private const LEVEL = 1;
}
