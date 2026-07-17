<?php

declare(strict_types=1);

final class DerivedFoo
{
    public string $name = '';
}

/**
 * @template T
 */
final class DerivedBox
{
    /** @var T */
    public mixed $value;

    /** @param T $value */
    public function __construct(mixed $value)
    {
        $this->value = $value;
    }
}

interface ParameterVariableDerivedTypes
{
    /**
     * @param array<array-key, object> $map
     * @return key-of<$map>
     */
    public function key(array $map): int|string;

    /**
     * @param array<array-key, object> $map
     * @return value-of<$map>
     */
    public function value(array $map): object;

    /**
     * @param array<array-key, object> $map
     * @return $map[$key]
     */
    public function get(array $map, int|string $key): object;

    /**
     * @param class-string $class
     * @return new<$class>
     */
    public function make(string $class): object;

    /** @return properties-of<$object> */
    public function properties(object $object): array;

    /**
     * @param class-string $class
     * @param non-empty-string $template
     * @return template-type<$box, $class, $template>
     */
    public function unbox(DerivedBox $box, string $class, string $template): mixed;

    /** @return int-mask<$first, $second> */
    public function mask(int $first, int $second): int;
}

function exercise_parameter_variable_derived_types(ParameterVariableDerivedTypes $types): void
{
    take_derived_foo_key($types->key(['foo' => new DerivedFoo()]));
    take_derived_foo($types->value(['foo' => new DerivedFoo()]));
    take_derived_foo($types->get(['foo' => new DerivedFoo()], 'foo'));
    take_derived_foo($types->make(DerivedFoo::class));
    take_derived_foo_properties($types->properties(new DerivedFoo()));

    $box = new DerivedBox(new DerivedFoo());
    take_derived_foo($types->unbox($box, DerivedBox::class, 'T'));

    take_derived_mask($types->mask(1, 2));
}

/** @param 'foo' $_ */
function take_derived_foo_key(string $_): void {}

function take_derived_foo(DerivedFoo $_): void {}

/** @param array{name: string} $_ */
function take_derived_foo_properties(array $_): void {}

/** @param 0|1|2|3 $_ */
function take_derived_mask(int $_): void {}
