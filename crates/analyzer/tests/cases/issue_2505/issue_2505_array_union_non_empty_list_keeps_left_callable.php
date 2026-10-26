<?php

final class NonEmptyListDuplicateCallbackFailure extends Exception
{
}

final class NonEmptyListDuplicateCallback
{
    /** @throws NonEmptyListDuplicateCallbackFailure */
    public static function stored(): void
    {
        throw new NonEmptyListDuplicateCallbackFailure();
    }
}

/** @param non-empty-list<callable(): void> $callbacks */
function preserve_non_empty_list_callback(array $callbacks): void
{
    $callbacks += [NonEmptyListDuplicateCallback::stored(...)];
}
