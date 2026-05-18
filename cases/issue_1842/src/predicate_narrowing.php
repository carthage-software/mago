<?php

declare(strict_types=1);

final class Service {}

function makeService(): Service
{
    return new Service();
}

function isService(mixed $value): bool
{
    return $value instanceof Service;
}

function run(mixed $thing): Service
{
    if (isService($thing)) {
        return $thing;
    }

    return makeService();
}
