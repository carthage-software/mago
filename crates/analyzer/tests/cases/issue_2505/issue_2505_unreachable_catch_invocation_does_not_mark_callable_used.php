<?php

final class Issue2505UnreachableCatchInvocationFailure extends Exception
{
}

function unreachable_catch_invocation_does_not_mark_callable_used(): void
{
    $callback = /** @throws Issue2505UnreachableCatchInvocationFailure */ static function (): void {
        throw new Issue2505UnreachableCatchInvocationFailure();
    };

    try {
    } catch (LogicException) { // @mago-expect analysis:dead-catch
        $callback();
    }
}
