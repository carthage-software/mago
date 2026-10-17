<?php

final class NonArrayAdditionFailure extends Exception
{
}

final class NonArrayAdditionCallback
{
    /** @throws NonArrayAdditionFailure */
    public static function stored(): void
    {
        throw new NonArrayAdditionFailure();
    }
}

function non_array_addition_does_not_store_callable(): void
{
    $value = 1;
    $value += NonArrayAdditionCallback::stored(...);
}

non_array_addition_does_not_store_callable();
