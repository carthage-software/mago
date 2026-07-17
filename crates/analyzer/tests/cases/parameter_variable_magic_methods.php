<?php

declare(strict_types=1);

final class MagicDependentFoo {}

final class MagicDependentBar {}

/**
 * @type MagicDependentObjects = array{foo: MagicDependentFoo, bar: MagicDependentBar}
 * @method MagicDependentObjects[$key] get(string $key)
 * @method static MagicDependentObjects[$key] staticGet(string $key)
 * @method void store(string $key, MagicDependentObjects[$key] $value)
 */
final class ParameterVariableMagicMethods
{
    public function __call(string $name, array $arguments): mixed
    {
        exit();
    }

    public static function __callStatic(string $name, array $arguments): mixed
    {
        exit();
    }
}

function exercise_parameter_variable_magic_methods(ParameterVariableMagicMethods $magic): void
{
    take_magic_dependent_foo($magic->get('foo'));
    take_magic_dependent_foo(ParameterVariableMagicMethods::staticGet('foo'));

    $magic->store('foo', new MagicDependentFoo());
    // @mago-expect analysis:invalid-argument
    $magic->store('foo', new MagicDependentBar());
}

function take_magic_dependent_foo(MagicDependentFoo $_): void {}
