<?php

declare(strict_types=1);

final class O
{
    public function __construct(
        public int $type,
    ) {}
}

function get(?O $o): O
{
    switch ($o?->type) {
        case 1:
            return $o;
        default:
            exit(1);
    }
}

function loose_not(?O $o): O
{
    // @mago-expect analysis:possibly-null-operand
    if ($o?->type != 1) {
        exit(1);
    }

    return $o;
}
