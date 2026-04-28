<?php

declare(strict_types=1);

final class Wrapper
{
    public null|string $value = null;
}

function flow_property_after_null_check(Wrapper $w): int
{
    if ($w->value === null) {
        return 0;
    }

    return strlen($w->value);
}
