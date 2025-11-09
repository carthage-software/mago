<?php

// Test: Multi-level template inheritance with interfaces
// This tests that template parameters are correctly substituted through
// multiple levels of interface inheritance

/**
 * @template T
 */
interface Comparable
{
    /**
     * @param T $other
     */
    public function compareTo(mixed $other): int;
}

/**
 * @extends Comparable<Foo>
 */
interface Foo extends Comparable
{
}

interface Bar extends Foo
{
}

// This trait incorrectly narrows the parameter type from Foo to Bar
// Expected: compareTo(Foo $other)
// Actual: compareTo(Bar $other)
// This violates LSP parameter contravariance
// @mago-expect analysis:incompatible-parameter-type
class BarImpl implements Bar
{
    /**
     * @param Bar $other
     */
    public function compareTo(mixed $other): int
    {
        return 0;
    }
}

// This is the correct implementation - parameter type matches the interface
class CorrectBarImpl implements Bar
{
    /**
     * @param Foo $other
     */
    public function compareTo(mixed $other): int
    {
        return 0;
    }
}
