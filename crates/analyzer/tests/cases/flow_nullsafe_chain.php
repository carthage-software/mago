<?php

declare(strict_types=1);

final class Inner
{
    public function name(): string
    {
        return 'inner';
    }
}

final class Outer
{
    public function __construct(public null|Inner $inner)
    {
    }
}

function flow_nullsafe_chain(null|Outer $o): null|string
{
    return $o?->inner?->name();
}
