<?php

declare(strict_types=1);

final class PartialBindingFoo {}

final class PartialBindingBar {}

/**
 * @type PartialBindingObjects = array{foo: PartialBindingFoo, bar: PartialBindingBar}
 */
interface ParameterVariablePartialMixedBindings
{
    /** @return list{PartialBindingObjects[$first], PartialBindingObjects[$second]} */
    public function pair(string $first, string $second): array;

    /** @param PartialBindingObjects[$key] $value */
    public function store(string $key, object $value): void;
}

function exercise_parameter_variable_partial_mixed_bindings(ParameterVariablePartialMixedBindings $types): void
{
    $pairWithFoo = $types->pair('foo', ?);
    take_partial_binding_pair($pairWithFoo('bar'));

    $storeFoo = $types->store('foo', ?);
    $storeFoo(new PartialBindingFoo());
    // @mago-expect analysis:invalid-argument
    $storeFoo(new PartialBindingBar());

    $storeValue = $types->store(?, new PartialBindingFoo());
    $storeValue('foo');

    $storeValueForBar = $types->store(?, new PartialBindingFoo());
    // @mago-expect analysis:invalid-argument
    $storeValueForBar('bar');
}

/** @param list{PartialBindingFoo, PartialBindingBar} $_ */
function take_partial_binding_pair(array $_): void {}
