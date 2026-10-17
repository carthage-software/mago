<?php

final class MixedListCallbackFailure extends Exception
{
}

final class MixedListCallback
{
    /** @throws MixedListCallbackFailure */
    public static function stored(): void
    {
        throw new MixedListCallbackFailure();
    }
}

/** @param list<callable(): void>|array<int, callable(): void> $callbacks */
function add_to_mixed_callback_list(array $callbacks): void
{
    $callbacks += [MixedListCallback::stored(...)];
}
