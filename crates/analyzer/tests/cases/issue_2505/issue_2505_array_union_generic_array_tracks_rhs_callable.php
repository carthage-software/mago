<?php

final class GenericIntegerArrayCallbackFailure extends Exception
{
}

final class GenericIntegerArrayCallback
{
    /** @throws GenericIntegerArrayCallbackFailure */
    public static function stored(): void
    {
        throw new GenericIntegerArrayCallbackFailure();
    }
}

/** @param array<array-key, mixed> $callbacks */
function add_to_generic_integer_callback_array(array $callbacks): void
{
    $callbacks += [GenericIntegerArrayCallback::stored(...)];
}
