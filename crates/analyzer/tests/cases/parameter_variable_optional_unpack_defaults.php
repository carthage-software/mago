<?php

declare(strict_types=1);

final class OptionalUnpackFoo {}

final class OptionalUnpackBar {}

interface ParameterVariableOptionalUnpackDefaults
{
    /**
     * @param array{foo: class-string<OptionalUnpackFoo>, bar: class-string<OptionalUnpackBar>} $classes
     * @param key-of<$classes> $key
     * @return new<$classes[$key]>
     */
    public function create(
        array $classes = ['foo' => OptionalUnpackFoo::class, 'bar' => OptionalUnpackBar::class],
        string $key = 'foo',
    ): object;
}

function exercise_parameter_variable_optional_unpack_defaults(
    ParameterVariableOptionalUnpackDefaults $types,
    bool $useDefault,
): void {
    $arguments = $useDefault ? [] : ['key' => 'bar'];
    $result = $types->create(...$arguments);

    // The empty branch returns OptionalUnpackFoo and the other returns OptionalUnpackBar.
    // @mago-expect analysis:possibly-invalid-argument
    take_optional_unpack_bar($result);
}

function take_optional_unpack_bar(OptionalUnpackBar $_): void {}
