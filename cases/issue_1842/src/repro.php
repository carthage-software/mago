<?php

declare(strict_types=1);

final class Foo {}

function supportsNormalization(mixed $data): bool
{
    return $data instanceof Foo;
}

$foo = new Foo();

if (supportsNormalization($foo)) {
    echo 'ok';
}
