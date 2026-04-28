<?php

declare(strict_types=1);

function callables_one_str(string $value): string
{
    return $value;
}

callables_one_str(value: 'ok', extra: 'no'); // @mago-expect analysis:too-many-arguments,invalid-named-argument
