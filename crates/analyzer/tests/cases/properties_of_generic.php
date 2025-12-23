<?php

declare(strict_types=1);

abstract class AbstractProperty
{
    /**
     * @template T of object
     *
     * @param T $object
     * @param key-of<properties-of<T>> $field
     */
    public static function isInitialized(object $object, string $field): bool
    {
        // @mago-expect analysis:unhandled-thrown-type
        $property = new ReflectionProperty($object::class, $field);

        return $property->isInitialized($object);
    }
}
