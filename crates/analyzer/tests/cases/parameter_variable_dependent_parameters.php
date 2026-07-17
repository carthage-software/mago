<?php

declare(strict_types=1);

final class ParameterFoo {}

final class ParameterBar {}

/**
 * @type Services = array{
 *   'foo.service': ParameterFoo,
 *   'bar.service': ParameterBar,
 * }
 */
interface ParameterVariableDependentParameters
{
    /** @param Services[$service] $value */
    public function set(string $service, mixed $value): void;

    /** @param list<Services[$service]> $values */
    public function setList(string $service, array $values): void;

    /** @param ($service is 'foo.service' ? ParameterFoo : ParameterBar) $value */
    public function setConditional(string $service, object $value): void;
}

function exercise_parameter_variable_dependent_parameters(ParameterVariableDependentParameters $parameters): void
{
    $parameters->set('foo.service', new ParameterFoo());
    // @mago-expect analysis:invalid-argument
    $parameters->set('foo.service', new ParameterBar());

    $parameters->setList('foo.service', [new ParameterFoo()]);
    // @mago-expect analysis:possibly-invalid-argument
    $parameters->setList('foo.service', [new ParameterBar()]);

    $parameters->setConditional('foo.service', new ParameterFoo());
    // @mago-expect analysis:invalid-argument
    $parameters->setConditional('foo.service', new ParameterBar());

    $parameters->set(...['service' => 'foo.service', 'value' => new ParameterFoo()]);
    // @mago-expect analysis:invalid-argument
    $parameters->set(...['service' => 'foo.service', 'value' => new ParameterBar()]);

    /**
     * @mago-expect analysis:invalid-argument
     */
    $parameters->set(value: new ParameterBar(), service: 'foo.service');
}
