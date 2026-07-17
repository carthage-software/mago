<?php

declare(strict_types=1);

final class InvocationFoo {}

final class InvocationBar {}

/**
 * @type Services = array{
 *   'foo.service': InvocationFoo,
 *   'bar.service': InvocationBar,
 * }
 */
interface ParameterVariableInvocationForms
{
    /** @return Services[$service] */
    public function get(string $service): object;

    /** @return Services[$service] */
    public function getDefault(string $service = 'foo.service'): object;

    /** @return Services[$service] */
    public function getWithExtra(string $service, int $extra): object;
}

function exercise_parameter_variable_invocation_forms(ParameterVariableInvocationForms $locator): void
{
    take_invocation_foo($locator->get('foo.service'));
    take_invocation_foo($locator->get(service: 'foo.service'));
    take_invocation_foo($locator->getDefault());
    take_invocation_bar($locator->getDefault(...['service' => 'bar.service']));

    $arrayCallable = [$locator, 'get'];
    take_invocation_foo($arrayCallable('foo.service'));
    take_invocation_foo($locator->get(...['service' => 'foo.service']));

    $firstClassCallable = $locator->get(...);
    take_invocation_foo($firstClassCallable('foo.service'));

    $partial = $locator->get(?);
    take_invocation_foo($partial('foo.service'));

    $boundPartial = $locator->getWithExtra('foo.service', ?);
    take_invocation_foo($boundPartial(1));
}

function take_invocation_foo(InvocationFoo $_): void {}

function take_invocation_bar(InvocationBar $_): void {}
