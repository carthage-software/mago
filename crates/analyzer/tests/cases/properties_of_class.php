<?php

/**
 * Tests for properties-of<T> with classes
 */

// Non-final class (produces unsealed array)
class User
{
    public string $name = '';
    public int $age = 0;
    protected string $_email = '';
    private string $_password = '';
}

// Final class (produces sealed array)
final class Config
{
    public string $key = '';
    public mixed $value = null;
}

// Class with inherited properties
class Admin extends User
{
    public bool $isSuper = false;
}

/**
 * properties-of<User> expands to:
 * array{name: string, age: int, _email: string, _password: string, ...}
 * (unsealed because User is not final - subclasses may have more properties)
 *
 * @param array{name: string, age: int, _email: string, _password: string, ...} $_
 */
function accepts_user_properties(array $_): void
{
}

/**
 * properties-of<Config> expands to:
 * array{key: string, value: mixed}
 * (sealed because Config is final)
 *
 * @param array{key: string, value: mixed} $_
 */
function accepts_config_properties(array $_): void
{
}

/**
 * public-properties-of<User> expands to:
 * array{name: string, age: int, ...}
 * (only public properties, unsealed)
 *
 * @param array{name: string, age: int, ...} $_
 */
function accepts_public_user_properties(array $_): void
{
}

/**
 * public-properties-of<Config> expands to:
 * array{key: string, value: mixed}
 * (only public properties, sealed because final)
 *
 * @param array{key: string, value: mixed} $_
 */
function accepts_public_config_properties(array $_): void
{
}

/**
 * @return properties-of<User>
 */
function get_user_properties(User $u): array
{
    return [
        'name' => $u->name,
        'age' => $u->age,
        '_email' => get_email($u),
        '_password' => get_password($u),
    ];
}

/**
 * @return properties-of<Config>
 */
function get_config_properties(Config $c): array
{
    return ['key' => $c->key, 'value' => $c->value];
}

/**
 * @return public-properties-of<User>
 */
function get_public_user_properties(User $u): array
{
    return ['name' => $u->name, 'age' => $u->age];
}

/**
 * @return public-properties-of<Config>
 */
function get_public_config_properties(Config $c): array
{
    return ['key' => $c->key, 'value' => $c->value];
}

// Helper functions to access protected/private properties
/** @return string */
function get_email(User $u): string
{
    return 'test@example.com';
}

/** @return string */
function get_password(User $u): string
{
    return 'hashed';
}

function test_user_properties(): void
{
    $user = new User();
    $props = get_user_properties($user);
    accepts_user_properties($props);
}

function test_config_properties(): void
{
    $config = new Config();
    $props = get_config_properties($config);
    accepts_config_properties($props);
}

function test_public_user_properties(): void
{
    $user = new User();
    $props = get_public_user_properties($user);
    accepts_public_user_properties($props);
}

function test_public_config_properties(): void
{
    $config = new Config();
    $props = get_public_config_properties($config);
    accepts_public_config_properties($props);
}
