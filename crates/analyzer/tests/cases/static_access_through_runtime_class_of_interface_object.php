<?php

interface Model
{
    public const KIND = 'model';

    public static function make(): static;
}

function through_get_class(Model $model): Model
{
    return get_class($model)::make();
}

function through_class_constant(Model $model): Model
{
    return $model::class::make();
}

function constant_through_runtime_class(Model $model): string
{
    return get_class($model)::KIND . $model::class::KIND;
}


/** @param class-string<Model> $class */
function from_class_string(string $class): Model
{
    // @mago-expect analysis:possibly-static-access-on-interface
    return $class::make();
}
