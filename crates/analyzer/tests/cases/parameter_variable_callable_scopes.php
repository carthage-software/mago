<?php

declare(strict_types=1);

final class CallableScopeFoo {}

final class CallableScopeBar {}

/**
 * @type CallableScopeObjects = array{foo: CallableScopeFoo, bar: CallableScopeBar}
 */
interface ParameterVariableCallableScopes
{
    /** @return callable(string $key): (CallableScopeObjects[$key]) */
    public function unbound(): callable;

    /** @return callable(string $key): (CallableScopeObjects[$key]) */
    public function shadow(string $key): callable;

    /** @return callable(string $key): (callable(string $ignored): (CallableScopeObjects[$key])) */
    public function curried(): callable;

    /** @return Closure(string $key): (CallableScopeObjects[$key]) */
    public function closure(): Closure;
}

function exercise_parameter_variable_callable_scopes(ParameterVariableCallableScopes $types): void
{
    $unbound = $types->unbound();
    take_callable_scope_foo($unbound('foo'));

    $shadowed = $types->shadow('bar');
    take_callable_scope_foo($shadowed('foo'));

    $curried = $types->curried();
    $forFoo = $curried('foo');
    take_callable_scope_foo($forFoo('ignored'));

    $closure = $types->closure();
    take_callable_scope_foo($closure('foo'));
}

function take_callable_scope_foo(CallableScopeFoo $_): void {}
