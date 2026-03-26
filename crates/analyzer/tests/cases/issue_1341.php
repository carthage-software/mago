<?php

declare(strict_types=1);

interface Foo {}

class A1341 implements Foo {}

class B1341 implements Foo {}

/**
 * @template T of Foo
 * @param class-string<T> $type
 */
function accept_type_1341(string $type): void {}

// Direct calls should work fine
accept_type_1341(A1341::class);
accept_type_1341(B1341::class);

// Union of class-strings should also work
$set = [A1341::class, B1341::class];
foreach ($set as $type) {
    accept_type_1341($type);
}
