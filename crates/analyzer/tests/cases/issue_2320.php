<?php

declare(strict_types=1);

/** @throws ReflectionException */
function short_name(string $class): string
{
    if (namespace\class_exists($class) || namespace\interface_exists($class)) {
        $class = new namespace\ReflectionClass($class)->getShortName();
    }

    return $class;
}
