<?php

declare(strict_types=1);

function callables_needs_string_only(string $s): int
{
    return strlen($s);
}

callables_needs_string_only(false);
