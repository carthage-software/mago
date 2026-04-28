<?php

declare(strict_types=1);

enum Color
{
    case Red;
    case Blue;
}

function flow_instanceof_enum(object $o): string
{
    if ($o instanceof Color) {
        return $o->name;
    }

    return 'other';
}
