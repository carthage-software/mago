<?php

declare(strict_types=1);

// On PHP >= 9.0, `null` default on a non-nullable parameter is a hard error:
// the implicit-nullable shim is removed.
// @mago-expect analysis:invalid-parameter-default-value
function implicit_null_removed(string $s = null): void
{
    unset($s);
}

// Explicit nullable stays valid.
function explicit_nullable(?string $s = null): void
{
    unset($s);
}
