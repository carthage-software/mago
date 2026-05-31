<?php

declare(strict_types=1);

/** @param class-string<object&callable> $class */
function test(string $class): void
{
    $callable = new $class(); // @mago-expect analysis:unknown-class-instantiation
    $callable();
}
