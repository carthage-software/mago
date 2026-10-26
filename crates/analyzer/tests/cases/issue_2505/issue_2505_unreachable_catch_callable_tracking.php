<?php

final class Issue2505DeadCatchFailure extends Exception
{
}

final class Issue2505DeadCatchCallback
{
    /** @throws Issue2505DeadCatchFailure */
    public static function thrower(): void
    {
        throw new Issue2505DeadCatchFailure();
    }
}

function unreachable_catch_storage_does_not_track_callable(): void
{
    try {
    } catch (LogicException) { // @mago-expect analysis:dead-catch
        $callback = Issue2505DeadCatchCallback::thrower(...);
    }
}

function unreachable_catch_invocation_does_not_track_callable(): void
{
    try {
    } catch (LogicException) { // @mago-expect analysis:dead-catch
        $callback = Issue2505DeadCatchCallback::thrower(...);
        $callback();
    }
}
