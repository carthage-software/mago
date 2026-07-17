<?php

declare(strict_types=1);

final class DirectParameterFoo {}

final class DirectParameterBar {}

interface ParameterVariableDirectParameterTypes
{
    /** @param $prototype $value */
    public function same(mixed $prototype, mixed $value): void;

    /** @param $prototype|int $value */
    public function sameOrInteger(mixed $prototype, mixed $value): void;

    /** @param $prototype $value */
    public function sameReversed(mixed $value, mixed $prototype): void;
}

function exercise_parameter_variable_direct_parameters(ParameterVariableDirectParameterTypes $types): void
{
    $types->same(new DirectParameterFoo(), new DirectParameterFoo());
    // @mago-expect analysis:invalid-argument
    $types->same(new DirectParameterFoo(), new DirectParameterBar());

    $types->sameOrInteger(new DirectParameterFoo(), new DirectParameterFoo());
    $types->sameOrInteger(new DirectParameterFoo(), 1);
    // @mago-expect analysis:invalid-argument
    $types->sameOrInteger(new DirectParameterFoo(), new DirectParameterBar());

    $types->sameReversed(new DirectParameterFoo(), new DirectParameterFoo());
    // @mago-expect analysis:invalid-argument
    $types->sameReversed(new DirectParameterBar(), new DirectParameterFoo());
}
