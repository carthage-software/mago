<?php

declare(strict_types=1);

final class NestedFoo {}

final class NestedBar {}

/**
 * @template T
 */
final class NestedBox
{
    /** @var T */
    public mixed $value;

    /** @param T $value */
    public function __construct(mixed $value)
    {
        $this->value = $value;
    }
}

/**
 * @type Services = array{
 *   'foo.service': NestedFoo,
 *   'bar.service': NestedBar,
 * }
 */
interface ParameterVariableNestedTypes
{
    /** @return list<Services[$service]> */
    public function getList(string $service): array;

    /** @return array<string, Services[$service]> */
    public function getArray(string $service): array;

    /** @return array{value: Services[$service]} */
    public function getShape(string $service): array;

    /** @return iterable<Services[$service]> */
    public function getIterable(string $service): iterable;

    /** @return NestedBox<Services[$service]> */
    public function getBox(string $service): NestedBox;

    /** @return callable(): (Services[$service]) */
    public function getCallable(string $service): callable;

    /** @return callable(Services[$service]): void */
    public function getConsumer(string $service): callable;

    /** @return class-string<Services[$service]> */
    public function getClass(string $service): string;

    /** @return object{value: Services[$service]} */
    public function getObjectShape(string $service): object;

    /** @return list<($value is NestedFoo ? NestedFoo : NestedBar)> */
    public function getConditionalList(object $value): array;

    /** @return array<($value is NestedFoo ? 'foo' : 'bar'), int> */
    public function getConditionalKeys(object $value): array;

    /** @param-out list<Services[$service]> $result */
    public function loadList(string $service, mixed &$result): void;

    /**
     * @param array<array-key, mixed> $values
     * @psalm-assert list<Services[$service]> $values
     */
    public function assertList(string $service, array $values): void;
}

function exercise_parameter_variable_nested_returns(ParameterVariableNestedTypes $types): void
{
    take_nested_foo_list($types->getList('foo.service'));
    take_nested_foo_array($types->getArray('foo.service'));
    take_nested_foo_shape($types->getShape('foo.service'));
    take_nested_foo_iterable($types->getIterable('foo.service'));
    take_nested_foo($types->getBox('foo.service')->value);
    take_nested_foo($types->getCallable('foo.service')());

    take_nested_foo_class($types->getClass('foo.service'));
    take_nested_foo_object_shape($types->getObjectShape('foo.service'));
    take_nested_foo_list($types->getConditionalList(new NestedFoo()));
    take_nested_foo_keys($types->getConditionalKeys(new NestedFoo()));

    $result = null;
    $types->loadList('foo.service', $result);
    take_nested_foo_list($result);
}

/** @param list<NestedFoo> $_ */
function take_nested_foo_list(array $_): void {}

/** @param array<string, NestedFoo> $_ */
function take_nested_foo_array(array $_): void {}

/** @param array{value: NestedFoo} $_ */
function take_nested_foo_shape(array $_): void {}

/** @param iterable<NestedFoo> $_ */
function take_nested_foo_iterable(iterable $_): void {}

function take_nested_foo(NestedFoo $_): void {}

/** @param class-string<NestedFoo> $_ */
function take_nested_foo_class(string $_): void {}

/** @param object{value: NestedFoo} $_ */
function take_nested_foo_object_shape(object $_): void {}

/** @param array<'foo', int> $_ */
function take_nested_foo_keys(array $_): void {}

/** @param array<array-key, mixed> $values */
function exercise_parameter_variable_nested_assertion(ParameterVariableNestedTypes $types, array $values): void
{
    $types->assertList('foo.service', $values);
    take_nested_foo_list($values);
}

function exercise_parameter_variable_nested_callable_parameter(ParameterVariableNestedTypes $types): void
{
    $consumer = $types->getConsumer('foo.service');
    $consumer(new NestedFoo());
    // @mago-expect analysis:invalid-argument
    $consumer(new NestedBar());
}
