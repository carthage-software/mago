<?php

declare(strict_types=1);

/**
 * @throws \DomainException
 */
function flow_match_with_throw_arm_default_declared(int $value): string
{
    return match ($value) {
        1 => 'one',
        2 => 'two',
        default => throw new \DomainException('unknown'),
    };
}

function flow_match_with_throw_arm_default_undeclared(int $value): string
{
    return match ($value) {
        1 => 'one',
        2 => 'two',
        default => throw new \DomainException('unknown'),
    };
}
