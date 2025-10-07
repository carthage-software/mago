<?php

declare(strict_types = 1);

const PATH = __DIR__  . '/../../Resources/';

class Foo
{
    public const PATH = __DIR__  . '/../../Resources/';
}

/**
 * @param non-empty-string $path
 *
 * @return array<int, string>
 */
function get_resources(string $path = PATH): array
{
    return scandir($path) ?: [];
}

var_dump(get_resources(PATH));
var_dump(get_resources(Foo::PATH));
