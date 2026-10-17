<?php

final class DeadInferredReturnFailure extends Exception
{
}

final class DeadInferredReturnCallback
{
    /** @throws DeadInferredReturnFailure */
    public static function stored(): void
    {
        throw new DeadInferredReturnFailure();
    }
}

function store_callback_from_unreachable_return(): void
{
    $make_callback = static function () {
        if (false) {
            return DeadInferredReturnCallback::stored(...);
        }

        return null;
    };

    $callback = $make_callback();
}
