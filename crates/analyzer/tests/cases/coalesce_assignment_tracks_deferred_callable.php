<?php

final class ProbeFailure extends Exception
{
}

/** @throws ProbeFailure */
function stored_target(): void
{
    throw new ProbeFailure(); // @mago-expect analysis:unhandled-thrown-type
}

function coalesce_assignment_tracks_deferred_callable(): void
{
    static $callback;
    $callback ??= stored_target(...);
}
