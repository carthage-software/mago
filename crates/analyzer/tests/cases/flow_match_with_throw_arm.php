<?php

declare(strict_types=1);

/**
 * @throws \DomainException
 */
function flow_match_with_throw_arm(int $v): string
{
    return match ($v) {
        1 => 'one',
        2 => 'two',
        default => throw new \DomainException('unknown'),
    };
}
