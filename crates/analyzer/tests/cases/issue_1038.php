<?php

declare(strict_types=1);

function test_no_default(): void
{
    /** @var string $field */
    $field = rand(min: 0, max: 1) ? 'a' : 'something_else';

    /** @var array<int>|null $options */
    $options = null;

    switch ($field) {
        case 'a':
            $options = [1, 2, 3];
            break;
    }

    if ($options !== null) {
        echo "options set\n";
    }
}

/**
 * @return list{1, 2, 3}
 *
 * @mago-expect analysis:unhandled-thrown-type
 */
function test_default_throws(): array
{
    /** @var string $field */
    $field = rand(min: 0, max: 1) ? 'a' : 'something_else';

    /** @var array<int>|null $options */
    $options = null;

    switch ($field) {
        case 'a':
            $options = [1, 2, 3];
            break;
        default:
            throw new \Exception('unexpected field');
    }

    return $options;
}

/**
 * @return false|array{1, 2, 3}
 */
function test_default_returns(): false|array
{
    /** @var string $field */
    $field = rand(min: 0, max: 1) ? 'a' : 'something_else';

    /** @var array<int>|null $options */
    $options = null;

    switch ($field) {
        case 'a':
            $options = [1, 2, 3];
            break;
        default:
            return false;
    }

    return $options;
}

/**
 * @return list{1, 2, 3}|list{4, 5, 6}
 */
function test_default_sets_variable(): array
{
    /** @var string $field */
    $field = rand(min: 0, max: 1) ? 'a' : 'something_else';

    /** @var array<int>|null $options */
    $options = null;

    switch ($field) {
        case 'a':
            $options = [1, 2, 3];
            break;
        default:
            $options = [4, 5, 6];
            break;
    }

    return $options;
}

/**
 * @return false|list{1, 2, 3}|list{4, 5, 6}
 */
function test_multiple_cases_no_default(): false|array
{
    /** @var string $field */
    $field = rand(min: 0, max: 2) ? (rand(0, 1) ? 'a' : 'b') : 'something_else';

    $options = null;

    switch ($field) {
        case 'a':
            $options = [1, 2, 3];
            break;
        case 'b':
            $options = [4, 5, 6];
            break;
    }

    if ($options !== null) {
        return $options;
    }

    return false;
}

function test_boolean_exhaustive(): string
{
    $flag = rand(0, 1) === 1;

    /** @var string|null $result */
    $result = null;

    switch ($flag) {
        case true:
            $result = 'yes';
            break;
        default:
            $result = 'no';
            break;
    }

    return $result;
}

function test_fallthrough(): void
{
    /** @var string $field */
    $field = rand(min: 0, max: 1) ? 'a' : 'something_else';

    /** @var array<int>|null $options */
    $options = null;

    switch ($field) {
        case 'a':
        case 'b':
            $options = [1, 2, 3];
            break;
    }

    if ($options !== null) {
        echo "options set\n";
    }
}
