<?php

declare(strict_types=1);

/**
 * @param array{name: string, age?: int} $person
 */
function test_empty_with_optional_key(array $person): void
{
    if (!empty($person['age'])) {
        echo $person['age'] + 1;
    }
}

/**
 * @param array{name: string, count?: int} $data
 */
function test_empty_branch_narrowing(array $data): int
{
    if (empty($data['count'])) {
        return 0;
    }

    return $data['count'];
}

/**
 * @param non-empty-string $str
 */
function test_empty_with_always_truthy(string $str): bool
{
    return empty($str);
}

/**
 * @param '' $str
 */
function test_empty_with_always_falsy(string $str): bool
{
    return empty($str);
}

function test_empty_with_null(?string $value): void
{
    if (!empty($value)) {
        echo strlen($value);
    }
}

/**
 * @param array<string, int> $map
 */
function test_empty_with_dynamic_key(array $map, string $key): void
{
    if (!empty($map[$key])) {
        echo $map[$key] + 1;
    }
}

/**
 * @param array{items?: list<string>} $data
 */
function test_empty_with_nested_array(array $data): void
{
    if (!empty($data['items'])) {
        foreach ($data['items'] as $item) {
            echo $item;
        }
    }
}

/**
 * @param array{config?: array{enabled?: bool}} $settings
 */
function test_empty_with_deeply_nested(array $settings): void
{
    if (!empty($settings['config'])) {
        $config = $settings['config'];
    }
}

class User
{
    public ?string $name = null;
    public int $age = 0;
}

function test_empty_with_object_property(User $user): void
{
    if (!empty($user->name)) {
        echo strlen($user->name);
    }
}

function test_empty_with_falsy_int(User $user): void
{
    if (!empty($user->age)) {
        echo $user->age / 2;
    }
}

/**
 * @param array{a: int, b?: string, c?: bool} $arr
 */
function test_empty_multiple_optional_keys(array $arr): string
{
    $result = '';

    if (!empty($arr['b'])) {
        $result .= $arr['b'];
    }

    if (!empty($arr['c'])) {
        $result .= 'yes';
    }

    return $result;
}

/**
 * @param list<array{id: int, meta?: array{tag?: string}}> $items
 */
function test_empty_in_loop(array $items): void
{
    foreach ($items as $item) {
        if (!empty($item['meta'])) {
            $meta = $item['meta'];

            if (!empty($meta['tag'])) {
                echo $meta['tag'];
            }
        }
    }
}

/**
 * @param array{value?: int|string} $data
 */
function test_empty_with_union_optional(array $data): void
{
    if (!empty($data['value'])) {
        echo $data['value'];
    }
}

function test_empty_ternary(?string $value): string
{
    return !empty($value) ? $value : 'default';
}

/**
 * @param array{required: int, optional?: int} $data
 */
function test_empty_does_not_affect_required_keys(array $data): void
{
    if (!empty($data['required'])) {
        echo $data['required'];
    }

    echo $data['required'];
}
