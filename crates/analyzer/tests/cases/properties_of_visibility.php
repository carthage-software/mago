<?php

/**
 * Tests for visibility-filtered properties-of<T> variants
 */

final class Entity
{
    public string $id;
    public string $name;
    protected string $internalId;
    protected int $version;
    private string $secret;
    private bool $dirty;
}

// Non-final class for unsealed array tests
class BaseModel
{
    public string $id;
    protected int $createdAt;
    private bool $initialized;
}

/**
 * public-properties-of<Entity> = array{id: string, name: string}
 *
 * @param array{id: string, name: string} $_
 */
function accepts_public_entity_props(array $_): void
{
}

/**
 * protected-properties-of<Entity> = array{internalId: string, version: int}
 *
 * @param array{internalId: string, version: int} $_
 */
function accepts_protected_entity_props(array $_): void
{
}

/**
 * private-properties-of<Entity> = array{secret: string, dirty: bool}
 *
 * @param array{secret: string, dirty: bool} $_
 */
function accepts_private_entity_props(array $_): void
{
}

/**
 * public-properties-of<BaseModel> = array{id: string, ...}
 * (unsealed because BaseModel is not final)
 *
 * @param array{id: string, ...} $_
 */
function accepts_public_base_model_props(array $_): void
{
}

/**
 * @return public-properties-of<Entity>
 */
function get_public_entity_props(): array
{
    return ['id' => 'uuid', 'name' => 'test'];
}

/**
 * @return protected-properties-of<Entity>
 */
function get_protected_entity_props(): array
{
    return ['internalId' => 'internal', 'version' => 1];
}

/**
 * @return private-properties-of<Entity>
 */
function get_private_entity_props(): array
{
    return ['secret' => 'shh', 'dirty' => false];
}

/**
 * @return public-properties-of<BaseModel>
 */
function get_public_base_model_props(): array
{
    return ['id' => 'uuid'];
}

function test_public_properties_of(): void
{
    $props = get_public_entity_props();
    accepts_public_entity_props($props);
}

function test_protected_properties_of(): void
{
    $props = get_protected_entity_props();
    accepts_protected_entity_props($props);
}

function test_private_properties_of(): void
{
    $props = get_private_entity_props();
    accepts_private_entity_props($props);
}

function test_public_base_model_properties_of(): void
{
    $props = get_public_base_model_props();
    accepts_public_base_model_props($props);
}
