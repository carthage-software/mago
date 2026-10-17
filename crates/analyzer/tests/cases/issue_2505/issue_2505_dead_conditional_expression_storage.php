<?php

final class DeadTernaryCallbackFailure extends Exception
{
}

final class DeadTernaryCallback
{
    /** @throws DeadTernaryCallbackFailure */
    public static function stored(): void
    {
        throw new DeadTernaryCallbackFailure();
    }
}

final class ConditionalTernaryCallbackFailure extends Exception
{
}

final class ConditionalTernaryCallback
{
    /** @throws ConditionalTernaryCallbackFailure */
    public static function stored(): void
    {
        throw new ConditionalTernaryCallbackFailure();
    }
}

function unreachable_ternary_assignment_does_not_store_callback(): void
{
    $callback = null;
    false ? ($callback = DeadTernaryCallback::stored(...)) : null;
}

function conditional_ternary_assignment_still_stores_callback(bool $condition): void
{
    $callback = null;
    $condition ? ($callback = ConditionalTernaryCallback::stored(...)) : null;
}
