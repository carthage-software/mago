<?php

declare(strict_types=1);

/**
 * @template T of object
 * @param class-string<T> $type
 * @return T
 */
function denormalize1372(mixed $data, string $type): object
{
    /** @var T - liar liar pants on fire */
    return new \stdClass();
}

/**
 * @template T of object
 * @param T $object
 * @return T
 */
function wrap1372(mixed $data, object $object): object
{
    $class = $object::class;
    $result = denormalize1372($data, $class);
    return $result;
}
