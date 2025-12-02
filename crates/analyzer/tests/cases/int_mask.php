<?php

class Permissions
{
    public const READ = 1;
    public const WRITE = 2;
    public const EXECUTE = 4;
}

class ExtendedPermissions
{
    public const PERM_READ = 1;
    public const PERM_WRITE = 2;
    public const PERM_EXECUTE = 4;
    public const OTHER = 'string';
}

/**
 * @param int-mask<1, 2, 4> $permissions
 */
function check_permissions(int $permissions): void
{
}

/**
 * @param int-mask-of<Permissions::*> $permissions
 */
function check_permissions_of(int $permissions): void
{
}

/**
 * @param int-mask-of<ExtendedPermissions::PERM_*> $permissions
 */
function check_permissions_prefix(int $permissions): void
{
}

/**
 * @return int-mask<1, 2, 4>
 */
function get_permissions(): int
{
    return 3;
}

/**
 * @return int-mask-of<Permissions::*>
 */
function get_permissions_of(): int
{
    return 5;
}

function test_int_mask_valid_values(): void
{
    check_permissions(0);
    check_permissions(1);
    check_permissions(2);
    check_permissions(3);
    check_permissions(4);
    check_permissions(5);
    check_permissions(6);
    check_permissions(7);
}

function test_int_mask_of_valid_values(): void
{
    check_permissions_of(0);
    check_permissions_of(1);
    check_permissions_of(2);
    check_permissions_of(3);
    check_permissions_of(4);
    check_permissions_of(5);
    check_permissions_of(6);
    check_permissions_of(7);
}

function test_int_mask_of_with_constants(): void
{
    check_permissions_of(Permissions::READ);
    check_permissions_of(Permissions::WRITE);
    check_permissions_of(Permissions::EXECUTE);
    check_permissions_of(Permissions::READ | Permissions::WRITE);
    check_permissions_of(Permissions::READ | Permissions::WRITE | Permissions::EXECUTE);
}

function test_int_mask_prefix_filter(): void
{
    check_permissions_prefix(0);
    check_permissions_prefix(1);
    check_permissions_prefix(2);
    check_permissions_prefix(3);
    check_permissions_prefix(4);
    check_permissions_prefix(5);
    check_permissions_prefix(6);
    check_permissions_prefix(7);
}

function test_return_type(): void
{
    $a = get_permissions();
    check_permissions($a);

    $b = get_permissions_of();
    check_permissions_of($b);
}

/**
 * @param int-mask<1, 2> $a
 * @param int-mask<4> $b
 * @return int-mask<1, 2, 4>
 */
function combine_permissions(int $a, int $b): int
{
    return $a | $b;
}

function test_combine(): void
{
    $combined = combine_permissions(1, 4);
    check_permissions($combined);
}

function test_int_mask_invalid_values(): void
{
    check_permissions(8); // @mago-expect analysis:invalid-argument
    check_permissions(9); // @mago-expect analysis:invalid-argument
    check_permissions(10); // @mago-expect analysis:invalid-argument
    check_permissions(100); // @mago-expect analysis:invalid-argument
    check_permissions(-1); // @mago-expect analysis:invalid-argument
}

function test_int_mask_of_invalid_values(): void
{
    check_permissions_of(8); // @mago-expect analysis:invalid-argument
    check_permissions_of(9); // @mago-expect analysis:invalid-argument
    check_permissions_of(16); // @mago-expect analysis:invalid-argument
    check_permissions_of(-1); // @mago-expect analysis:invalid-argument
}

function test_int_mask_prefix_invalid_values(): void
{
    check_permissions_prefix(8); // @mago-expect analysis:invalid-argument
    check_permissions_prefix(15); // @mago-expect analysis:invalid-argument
    check_permissions_prefix(255); // @mago-expect analysis:invalid-argument
}
