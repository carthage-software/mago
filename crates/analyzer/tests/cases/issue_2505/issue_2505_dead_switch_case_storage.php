<?php

final class DeadSwitchCaseFailure extends Exception
{
}

final class DeadSwitchCaseCallback
{
    /** @throws DeadSwitchCaseFailure */
    public static function stored(): void
    {
        throw new DeadSwitchCaseFailure();
    }
}

function unreachable_switch_case_does_not_store_callback(): void
{
    $callback = null;
    switch (true) {
        case false:
            $callback = DeadSwitchCaseCallback::stored(...);
            break;
        default:
            break;
    }
}
