<?php

declare(strict_types=1);

function flow_in_array_narrow(string $v): bool
{
    return in_array($v, ['a', 'b', 'c'], true);
}
