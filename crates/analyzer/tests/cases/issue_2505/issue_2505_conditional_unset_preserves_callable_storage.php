<?php

final class ConditionalUnsetFailure extends Exception
{
}

final class ConditionalUnsetCallback
{
    /** @throws ConditionalUnsetFailure */
    public static function stored(): void
    {
        throw new ConditionalUnsetFailure();
    }
}

function conditional_unset_preserves_callable_storage(bool $should_unset): void
{
    $callbacks = [];
    $callbacks['job'] = ConditionalUnsetCallback::stored(...);

    if ($should_unset) {
        unset($callbacks);
    }
}
