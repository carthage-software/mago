<?php

// Test case for array key access after existence checks
//
// After checking that a key exists (with isset, array_key_exists, or null coalesce),
// the subsequent access should not trigger undefined key warnings.

/**
 * @param array<string, mixed> $data
 */
function processWithIsset(array $data): mixed
{
    if (isset($data['key'])) {
        // After isset check, accessing $data['key'] should be safe
        return $data['key'];
    }

    return null;
}

/**
 * @param array<string, mixed> $config
 */
function processWithArrayKeyExists(array $config): string
{
    if (array_key_exists('name', $config)) {
        // After array_key_exists, the key definitely exists
        return (string) $config['name'];
    }

    return 'default';
}

/**
 * @param array<string, mixed> $options
 */
function processMultipleKeys(array $options): array
{
    $result = [];

    if (isset($options['foo'], $options['bar'])) {
        // Both keys exist after this check
        $result['foo'] = $options['foo'];
        $result['bar'] = $options['bar'];
    }

    return $result;
}

/**
 * @param array<string, mixed> $data
 */
function nestedArrayAccess(array $data): null|string
{
    if (isset($data['user']['name'])) {
        // Nested access should be safe after isset
        return (string) $data['user']['name'];
    }

    return null;
}

processWithIsset(['key' => 'value']);
processWithArrayKeyExists(['name' => 'test']);
processMultipleKeys(['foo' => 1, 'bar' => 2]);
nestedArrayAccess(['user' => ['name' => 'John']]);
