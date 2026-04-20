<?php

function get_val(): mixed
{
    return 1;
}

class Example
{
}

class ExampleWithActualMethods
{
    public function foo(): mixed
    {
        return get_val();
    }

    public static function bar(): mixed
    {
        return get_val();
    }
}

class ExampleWithMagicMethods
{
    /** @param array<mixed> $arguments */
    public function __call(string $name, array $arguments): mixed
    {
        return get_val();
    }

    /** @param array<mixed> $arguments */
    public static function __callStatic(string $name, array $arguments): mixed
    {
        return get_val();
    }
}

/**
 * @method void foo()
 * @method static void bar()
 */
final class ExampleWithMagicTags
{
}

/**
 * @method void foo()
 * @method static void bar()
 */
class ExampleWithBoth
{
    /** @param array<mixed> $arguments */
    public function __call(string $name, array $arguments): mixed
    {
        return get_val();
    }

    /** @param array<mixed> $arguments */
    public static function __callStatic(string $name, array $arguments): mixed
    {
        return get_val();
    }
}

/**
 * @method static void bar()
 */
class ExampleWithBothButStaticOnly
{
    /** @param array<mixed> $arguments */
    public static function __callStatic(string $name, array $arguments): mixed
    {
        return get_val();
    }
}

function error(): void
{
    (new Example())->foo(); // @mago-expect analysis:non-existent-method
    Example::bar(); // @mago-expect analysis:non-existent-method
    (new ExampleWithBothButStaticOnly())->bar(); // @mago-expect analysis:dynamic-static-method-call
    (new ExampleWithBoth())->bar(); // @mago-expect analysis:dynamic-static-method-call
}

function warn(): void
{
    (new ExampleWithMagicMethods())->foo(); // @mago-expect analysis:non-documented-method
    ExampleWithMagicMethods::bar(); // @mago-expect analysis:non-documented-method
    (new ExampleWithMagicTags())->foo(); // @mago-expect analysis:missing-magic-method
    ExampleWithMagicTags::bar(); // @mago-expect analysis:missing-magic-method
}

function ok(): void
{
    (new ExampleWithActualMethods())->foo(); // ok
    ExampleWithActualMethods::bar(); // ok
    (new ExampleWithBoth())->foo(); // ok
    ExampleWithBoth::bar(); // ok
    ExampleWithBothButStaticOnly::bar(); // ok
}
