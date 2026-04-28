<?php

declare(strict_types=1);

function flow_is_resource_narrow(mixed $v): bool
{
    if (is_resource($v)) {
        return true;
    }

    return false;
}
