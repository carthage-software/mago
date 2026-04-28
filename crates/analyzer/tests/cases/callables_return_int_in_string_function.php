<?php

declare(strict_types=1);

function callables_returns_string_only(): string
{
    /** @mago-expect analysis:invalid-return-statement */
    return 42;
}

echo callables_returns_string_only();
