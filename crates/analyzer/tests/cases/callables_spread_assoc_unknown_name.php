<?php

declare(strict_types=1);

function callables_named_target(string $first, int $second): string
{
    return $first . $second;
}

callables_named_target(...['first' => 'a', 'second' => 1, 'third' => 'no']); // @mago-expect analysis:too-many-arguments,invalid-named-argument
