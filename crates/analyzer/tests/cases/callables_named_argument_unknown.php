<?php

declare(strict_types=1);

function callables_two_named(string $name, int $count): string
{
    return $name . $count;
}

callables_two_named(name: 'x', count: 1, extra: 'no'); // @mago-expect analysis:too-many-arguments,invalid-named-argument
