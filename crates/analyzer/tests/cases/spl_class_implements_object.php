<?php

declare(strict_types=1);

function interfacesOfObject(object $object): array
{
    return array_values(class_implements($object));
}

/**
 * @param class-string $class
 */
function interfacesOfUnknownString(string $class): array
{
    /** @mago-expect analysis:possibly-false-argument */
    return array_values(class_implements($class));
}

/**
 * @param object|class-string $value
 */
function interfacesOfMaybeObject(object|string $value): array
{
    /** @mago-expect analysis:possibly-false-argument */
    return array_values(class_implements($value));
}
