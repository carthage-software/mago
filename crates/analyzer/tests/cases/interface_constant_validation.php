<?php

// Test 1: Interface constant cannot be overridden with different value
interface InterfaceWithConstant
{
    public const VALUE = 100;
}

class ClassOverridesInterfaceConstant implements InterfaceWithConstant
{
    // Valid: PHP allows overriding interface constant values
    public const VALUE = 200;
}

// Test 2: Interface constant must be public in implementing class
interface InterfacePublicConstant
{
    public const NAME = 'interface';
}

class ClassNarrowsInterfaceConstantToProtected implements InterfacePublicConstant
{
    // @mago-expect analysis:incompatible-constant-visibility
    protected const NAME = 'interface';
}

// Test 3: Interface constant must be public (private violation)
interface InterfacePublicConstant2
{
    public const NAME = 'test';
}

class ClassNarrowsInterfaceConstantToPrivate implements InterfacePublicConstant2
{
    // @mago-expect analysis:incompatible-constant-visibility
    private const NAME = 'test';
}

// Test 4: Valid implementation - same value, public visibility
interface InterfaceValidConstant
{
    public const CODE = 42;
}

class ValidInterfaceImplementation implements InterfaceValidConstant
{
    public const CODE = 42;
}

// Test 5: Not declaring the constant is fine (inherited)
interface InterfaceInheritedConstant
{
    public const STATUS = 'active';
}

class ClassInheritsConstant implements InterfaceInheritedConstant
{
    // No constant declaration - should inherit from interface
}

// Test 6: Multiple interfaces
interface InterfaceA
{
    public const A = 1;
}

interface InterfaceB
{
    public const B = 2;
}

class ClassImplementsMultiple implements InterfaceA, InterfaceB
{
    public const A = 1;
    public const B = 2;
}

// Test 7: Multi-level interface inheritance
interface GrandInterface
{
    public const VALUE = 'grand';
}

interface ParentInterface extends GrandInterface
{
}

class ChildImplementsParent implements ParentInterface
{
    // Valid: PHP allows overriding interface constant values through inheritance
    public const VALUE = 'child';
}

// Test 8: Valid - same value through inheritance chain
interface BaseInterface
{
    public const ID = 123;
}

interface ExtendedInterface extends BaseInterface
{
}

class ImplementsExtended implements ExtendedInterface
{
    public const ID = 123;
}
