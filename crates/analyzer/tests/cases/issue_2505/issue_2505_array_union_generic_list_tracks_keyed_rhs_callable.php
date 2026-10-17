<?php

final class GenericListKeyedCallbackFailure extends Exception
{
}

final class GenericListKeyedCallback
{
    /** @throws GenericListKeyedCallbackFailure */
    public static function stored(): void
    {
        throw new GenericListKeyedCallbackFailure();
    }
}

/** @param list<callable(): void> $callbacks */
function add_keyed_rhs_to_generic_callback_list(array $callbacks): void
{
    $callbacks += [0 => GenericListKeyedCallback::stored(...), 'metadata' => 'value'];
}

add_keyed_rhs_to_generic_callback_list([]);
