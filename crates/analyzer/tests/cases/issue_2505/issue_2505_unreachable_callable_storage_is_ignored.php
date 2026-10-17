<?php

final class DeadAssignmentFailure extends Exception
{
}

final class DeadAssignmentCallback
{
    /** @throws DeadAssignmentFailure */
    public static function stored(): void
    {
        throw new DeadAssignmentFailure();
    }
}

final class DeadCoalesceAssignmentFailure extends Exception
{
}

final class DeadCoalesceAssignmentCallback
{
    /** @throws DeadCoalesceAssignmentFailure */
    public static function stored(): void
    {
        throw new DeadCoalesceAssignmentFailure();
    }
}

final class ConditionalAssignmentFailure extends Exception
{
}

final class ConditionalAssignmentCallback
{
    /** @throws ConditionalAssignmentFailure */
    public static function stored(): void
    {
        throw new ConditionalAssignmentFailure();
    }
}

final class ConditionalCoalesceAssignmentFailure extends Exception
{
}

final class ConditionalCoalesceAssignmentCallback
{
    /** @throws ConditionalCoalesceAssignmentFailure */
    public static function stored(): void
    {
        throw new ConditionalCoalesceAssignmentFailure();
    }
}

final class DeadReturnFailure extends Exception
{
}

final class DeadReturnCallback
{
    /** @throws DeadReturnFailure */
    public static function stored(): void
    {
        throw new DeadReturnFailure();
    }
}

function unreachable_assignment_does_not_store_callback(): void
{
    $callback = null;
    if (false) {
        $callback = DeadAssignmentCallback::stored(...);
    }
}

function unreachable_coalesce_assignment_does_not_store_callback(?Closure $callback): void
{
    if (false) {
        $callback ??= DeadCoalesceAssignmentCallback::stored(...);
    }
}

function conditional_assignment_still_stores_callback(bool $condition): void
{
    $callback = null;
    if ($condition) {
        $callback = ConditionalAssignmentCallback::stored(...);
    }
}

function conditional_coalesce_assignment_still_stores_callback(bool $condition, ?Closure $callback): void
{
    if ($condition) {
        $callback ??= ConditionalCoalesceAssignmentCallback::stored(...);
    }
}

function unreachable_return_does_not_store_callback(): ?Closure
{
    if (false) {
        return DeadReturnCallback::stored(...);
    }

    return null;
}
