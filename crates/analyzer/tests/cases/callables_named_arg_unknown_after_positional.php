<?php

declare(strict_types=1);

function callables_two_args(string $a, int $b): string
{
    return $a . $b;
}

callables_two_args('x', wrongName: 1); // @mago-expect analysis:invalid-named-argument
