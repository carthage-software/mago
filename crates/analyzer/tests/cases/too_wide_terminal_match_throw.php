<?php

declare(strict_types=1);

/**
 * @throws \DomainException
 */
function too_wide_terminal_match_throw_declared(bool $value): string
{
    return match ($value) {
        true => throw new \DomainException('unknown'),
        false => 'one',
    };
}

/**
 * @throws \RuntimeException
 */
function too_wide_terminal_match_throw_mismatch(bool $value): string
{
    return match ($value) {
        true => throw new \DomainException('unknown'),
        false => 'one',
    };
}

/**
 * @throws \DomainException
 */
function too_wide_terminal_match_throw_caught(bool $value): string
{
    try {
        return match ($value) {
            true => throw new \DomainException('unknown'),
            false => 'one',
        };
    } catch (\DomainException) {
        return 'caught';
    }
}
