<?php

final class OtherScopeUnsetFailure extends Exception
{
}

final class OtherScopeUnsetCallback
{
    /** @throws OtherScopeUnsetFailure */
    public static function stored(): void
    {
        throw new OtherScopeUnsetFailure();
    }
}

function store_callable_in_original_scope(): void
{
    $callbacks = [];
    $callbacks['job'] = OtherScopeUnsetCallback::stored(...);
}

function unset_same_variable_in_another_scope(): void
{
    $callbacks = [];
    unset($callbacks);
}
