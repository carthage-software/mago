<?php

declare(strict_types=1);

enum Type
{
    case One;
    case Two;
    case Three;
}

/**
 * @return ($type is Type::One ? array{key1: string} : array{key2: string})
 */
function test(Type $type): mixed
{
    if (Type::One === $type) {
        return ['key1' => 'a'];
    }

    return ['key2' => 'a'];
}

/** @param array{key1: string} $_ */
function a(array $_): void {}

/** @param array{key2: string} $_ */
function b(array $_): void {}

a(test(Type::One));
b(test(Type::Two));
b(test(Type::Three));
