<?php

declare(strict_types=1);

final class SuppressProfile
{
    public function __construct(public readonly string $firstName = '')
    {
    }
}

final class SuppressItem
{
    public function title(): string
    {
        return 'title';
    }
}

/**
 * @mago-expect analysis:redundant-nullsafe-operator
 */
function coalesce_left(?SuppressProfile $profile): string
{
    return $profile?->firstName ?? '';
}

/**
 * @mago-expect analysis:redundant-nullsafe-operator
 */
function inside_isset(?SuppressProfile $profile): bool
{
    return isset($profile?->firstName);
}

/**
 * @mago-expect analysis:redundant-nullsafe-operator
 */
function inside_empty(?SuppressProfile $profile): bool
{
    return empty($profile?->firstName);
}

/**
 * @mago-expect analysis:redundant-nullsafe-operator
 */
function method_on_non_null(SuppressItem $item): string
{
    return $item?->title();
}

function method_on_nullable_left_of_coalesce_is_needed(?SuppressItem $item): string
{
    return $item?->title() ?? '';
}
