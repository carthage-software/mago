<?php

declare(strict_types=1);

final class PropertiesOfWithoutInstanceProperties
{
    public static int $count = 0;
}

final class PropertiesOfUntyped
{
    public $first = 1;
}

final class PropertiesOfMixed
{
    public int $typed = 1;
    public $untyped = 1;
}

/**
 * @param properties-of<PropertiesOfWithoutInstanceProperties> $properties
 */
function properties_of_class_without_instance_properties(array $properties): int
{
    return count($properties);
}

/**
 * @param properties-of<PropertiesOfUntyped> $properties
 */
function properties_of_untyped_class(array $properties): mixed
{
    return $properties['first'];
}

/**
 * @param properties-of<PropertiesOfMixed> $properties
 */
function properties_of_mixed_class(array $properties): int
{
    echo gettype($properties['untyped']);

    return $properties['typed'];
}
