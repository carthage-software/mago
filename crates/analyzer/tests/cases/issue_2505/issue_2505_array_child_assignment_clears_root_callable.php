<?php

final class RootArrayCallableFailure extends Exception
{
}

final class RootArrayCallable
{
    /** @throws RootArrayCallableFailure */
    public static function callback(): void
    {
        throw new RootArrayCallableFailure();
    }
}

function replacing_child_in_root_array_clears_callable(): void
{
    $callbacks = ['job' => RootArrayCallable::callback(...)];
    $callbacks['job'] = null;
}

function unsetting_child_in_root_array_clears_callable(): void
{
    $callbacks = ['job' => RootArrayCallable::callback(...)];
    unset($callbacks['job']);
}
