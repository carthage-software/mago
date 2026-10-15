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

final class SuppressOuter
{
    public function inner(): SuppressItem
    {
        return new SuppressItem();
    }
}

final class SuppressOuterProp
{
    public SuppressProfile $profile;

    public function __construct()
    {
        $this->profile = new SuppressProfile();
    }
}

/**
 * @mago-expect analysis:redundant-nullsafe-operator
 */
function property_on_non_null(SuppressProfile $profile): string
{
    return $profile?->firstName;
}

function coalesce_left(?SuppressProfile $profile): string
{
    return $profile?->firstName ?? '';
}

function inside_isset(?SuppressProfile $profile): bool
{
    return isset($profile?->firstName);
}

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

function chained_nullsafe_with_non_nullable_method_return(?SuppressOuter $outer): ?string
{
    return $outer?->inner()?->title();
}

function chained_nullsafe_with_non_nullable_prop(?SuppressOuterProp $outer): ?string
{
    return $outer?->profile?->firstName;
}
