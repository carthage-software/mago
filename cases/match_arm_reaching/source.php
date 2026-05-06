<?php

enum Status
{
    case Pending;
    case Active;
    case Inactive;
}

/**
 */
function test_unreachable_arm(Status $status): string
{
    return match ($status) {
        Status::Pending => 'pending',
        Status::Active => 'active',
        Status::Inactive => 'inactive', // Always true!
        Status::Pending => 'pending again?', // This arm is unreachable
    };
}

/**
 */
function test_unreachable_default(Status $status): string
{
    return match ($status) {
        Status::Pending => 'pending',
        Status::Active => 'active',
        Status::Inactive => 'inactive', // Always true
        default => 'unreachable', // All enum cases are covered
    };
}

/**
 * @param 'a'|'b' $letter
 *
 */
function test_always_matching_arm(string $letter): string
{
    return match ($letter) {
        'a' => 'is a',
        'b' => 'is b', // This arm is always true, as it's the only remaining possibility
        'c' => 'is c', // This arm is unreachable
    };
}

/**
 * A valid, exhaustive match that should produce no errors.
 */
function test_valid_exhaustive_match(Status $status): string
{
    return match ($status) {
        Status::Pending => 'pending',
        Status::Active => 'active',
        Status::Inactive => 'inactive',
    };
}
