<?php

declare(strict_types=1);

function flow_is_callable_narrow(mixed $v): mixed
{
    if (is_callable($v)) {
        return $v();
    }

    return null;
}
