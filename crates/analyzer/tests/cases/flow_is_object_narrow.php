<?php

declare(strict_types=1);

final class Widget
{
    public int $value = 0;
}

function flow_is_object_narrow(Widget|int $value): int
{
    if (is_object($value)) {
        return $value->value;
    }

    return $value + 1;
}
