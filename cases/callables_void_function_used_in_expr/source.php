<?php

declare(strict_types=1);

function callables_returns_void(): void
{
    echo 'side effect';
}

callables_returns_void();
