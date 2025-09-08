<?php

enum Status
{
    case Pending;
    case Active;
    case Inactive;
}

class Tag
{
    public function __construct(
        public string $name,
    ) {}
}

function get_bool(): bool
{
    return get_bool();
}

/**
 * @mago-expect analysis:empty-match-expression
 * @mago-expect analysis:unhandled-thrown-type
 */
function test_empty_match(Status $status): never
{
    match ($status) { };
}

/**
 * @mago-expect analysis:match-expression-only-default-arm
 */
function test_only_default(string $value): string
{
    return match ($value) { default => 'default value' };
}

/**
 * @mago-expect analysis:match-subject-type-is-never
 */
function test_subject_is_never(): never
{
    match (exit(1)) {
        1 => 'one',
        default => 'default',
    };
}

/**
 * @mago-expect analysis:match-arm-always-true
 * @mago-expect analysis:unreachable-match-arm
 */
function test_unreachable_arm_by_type(Status $status): string
{
    return match ($status) {
        Status::Pending => 'pending',
        Status::Active => 'active',
        Status::Inactive => 'inactive',
        Status::Pending => 'pending again?',
    };
}

/**
 * @mago-expect analysis:match-not-exhaustive
 * @mago-expect analysis:unreachable-match-arm
 * @mago-expect analysis:unhandled-thrown-type
 */
function test_unreachable_arm_by_condition(string $value): string
{
    return match ($value) {
        'a' => 'is a',
        'b' => 'is b',
        'c' => 'is c',
        1 => 'is 1',
    };
}

/**
 * @mago-expect analysis:match-arm-always-true
 * @mago-expect analysis:unreachable-match-default-arm
 */
function test_unreachable_default(Status $status): string
{
    return match ($status) {
        Status::Pending => 'pending',
        Status::Active => 'active',
        Status::Inactive => 'inactive',
        default => 'unreachable',
    };
}

/**
 * @mago-expect analysis:match-arm-always-true
 * @mago-expect analysis:unreachable-match-default-arm
 */
function test_always_matching_arm(bool $flag): string
{
    return match ($flag) {
        true => 'is true',
        false => 'is false',
        default => 'unreachable',
    };
}

/**
 * @mago-expect analysis:unreachable-match-arm
 * @mago-expect analysis:match-default-arm-always-executed
 */
function test_always_executed_default(string $s): string
{
    return match ($s) {
        1 => 'one',
        2 => 'two',
        default => 'always default',
    };
}

/**
 * @mago-expect analysis:match-not-exhaustive
 * @mago-expect analysis:unhandled-thrown-type
 */
function test_non_exhaustive(Status $status): string
{
    return match ($status) {
        Status::Pending => 'pending',
        Status::Active => 'active',
    };
}

function test_happy_path_exhaustive_enum(Status $status): string
{
    return match ($status) {
        Status::Pending => 'pending',
        Status::Active => 'active',
        Status::Inactive => 'inactive',
    };
}

function test_happy_path_with_default(int $value): string
{
    return match ($value) {
        1 => 'one',
        2 => 'two',
        default => 'many',
    };
}

function test_happy_path_complex_logic(): string
{
    $is_user_administrator = get_bool();
    $is_user_logged_in = get_bool();

    return match (true) {
        !$is_user_logged_in => 'User not logged in.',
        !$is_user_administrator => 'User is not an admin.',
        default => 'User is an admin!',
    };
}

function test_happy_path_complex_logic_no_default(): string
{
    $is_user_administrator = get_bool();
    $is_user_logged_in = get_bool();

    return match (true) {
        !$is_user_logged_in => 'User not logged in.',
        !$is_user_administrator => 'User is not an admin.',
        $is_user_administrator => 'User is an admin!',
    };
}

function test_happy_path_instance(string|Tag|null $tag): string
{
    return match (true) {
        is_string($tag) => $tag,
        $tag instanceof Tag => $tag->name,
        default => 'default',
    };
}
