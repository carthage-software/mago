<?php

declare(strict_types=1);

namespace MagicCallForInaccessibleMethod;

final class Factory
{
    public function visible(): string
    {
        return $this->hidden() . self::make() . self::build();
    }

    private function hidden(): string
    {
        return 'hidden';
    }

    private static function make(): string
    {
        return 'private';
    }

    protected static function build(): string
    {
        return 'protected';
    }

    /** @param list<mixed> $arguments */
    public function __call(string $name, array $arguments): string
    {
        return $name;
    }

    /** @param list<mixed> $arguments */
    public static function __callStatic(string $name, array $arguments): string
    {
        return $name;
    }
}

function instance_call(Factory $factory): string
{
    // @mago-expect analysis:non-documented-method
    return $factory->hidden();
}

function private_static_call(): string
{
    // @mago-expect analysis:non-documented-method
    return Factory::make();
}

function protected_static_call(): string
{
    // @mago-expect analysis:non-documented-method
    return Factory::build();
}

function visible_call(Factory $factory): string
{
    return $factory->visible();
}

final class Plain
{
    public function visible(): string
    {
        return $this->hidden() . self::make();
    }

    private function hidden(): string
    {
        return 'hidden';
    }

    private static function make(): string
    {
        return 'private';
    }
}

function plain_instance_call(Plain $plain): string
{
    // @mago-expect analysis:invalid-method-access
    return $plain->hidden();
}

function plain_static_call(): string
{
    // @mago-expect analysis:invalid-method-access
    return Plain::make();
}

abstract class StaticBase
{
    /** @param list<mixed> $arguments */
    public static function __callStatic(string $name, array $arguments): string
    {
        return $name;
    }
}

final class StaticChild extends StaticBase
{
    public function run(): string
    {
        // @mago-expect analysis:non-documented-method
        return parent::anything();
    }

    public static function runStatic(): string
    {
        // @mago-expect analysis:non-documented-method
        return parent::anything();
    }
}

abstract class InstanceBase
{
    /** @param list<mixed> $arguments */
    public function __call(string $name, array $arguments): string
    {
        return $name;
    }
}

final class InstanceChild extends InstanceBase
{
    public function viaParent(): string
    {
        // @mago-expect analysis:non-documented-method
        return parent::anything();
    }

    public function viaSelf(): string
    {
        // @mago-expect analysis:non-documented-method
        return self::anything();
    }

    public function viaStatic(): string
    {
        // @mago-expect analysis:non-documented-method
        return static::anything();
    }

    public static function fromStaticScope(): string
    {
        // @mago-expect analysis:non-existent-method
        // @mago-expect analysis:mixed-return-statement
        return self::anything();
    }
}
