<?php

declare(strict_types=1);

final class Inner
{
    public null|string $name = null;
}

final class Outer
{
    public null|Inner $inner = null;
}

function flow_property_isset_chain(Outer $o): string
{
    if (isset($o->inner) && isset($o->inner->name)) {
        return $o->inner->name;
    }

    return '';
}
