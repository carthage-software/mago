<?php

final class UnreachableUnsetFailure extends Exception
{
}

final class UnreachableUnsetCallback
{
    /** @throws UnreachableUnsetFailure */
    public static function stored(): void
    {
        throw new UnreachableUnsetFailure();
    }
}

function unreachable_unset_preserves_callable_storage(): void
{
    $callbacks = [];
    $callbacks['job'] = UnreachableUnsetCallback::stored(...);

    if (false) {
        unset($callbacks);
    }
}
