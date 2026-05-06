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
    public function __construct(
        public ?Inner $inner,
    ) {}
}

function flow_nullsafe_chain(?Outer $o): ?string
{
    return $o?->inner?->name();
}
