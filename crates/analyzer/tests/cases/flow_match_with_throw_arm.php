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

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function flow_match_with_unhandled_throw_arm(int $v): string
{
    return match ($v) {
        1 => 'one',
        2 => 'two',
        default => throw new \DomainException('unknown'),
    };
}
