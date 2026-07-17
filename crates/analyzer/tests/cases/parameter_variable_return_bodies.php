<?php

declare(strict_types=1);

final class ReturnBodyFoo {}

/** @return $value */
function parameter_variable_function_identity(mixed $value): mixed
{
    return $value;
}

final class ParameterVariableReturnBodies
{
    /** @return $value */
    public function identity(mixed $value): mixed
    {
        return $value;
    }

    /** @return $value */
    public static function staticIdentity(mixed $value): mixed
    {
        return $value;
    }
}

function exercise_parameter_variable_return_bodies(ParameterVariableReturnBodies $identities): void
{
    take_return_body_foo(parameter_variable_function_identity(new ReturnBodyFoo()));
    take_return_body_foo($identities->identity(new ReturnBodyFoo()));
    take_return_body_foo(ParameterVariableReturnBodies::staticIdentity(new ReturnBodyFoo()));
}

function take_return_body_foo(ReturnBodyFoo $_): void {}
