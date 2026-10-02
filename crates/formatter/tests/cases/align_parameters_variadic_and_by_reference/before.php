<?php

final class Variadic
{
    public function __construct(
        public readonly int $priority = 0,
        string ...$renders,
    ) {}
}

final class ByReference
{
    public function __construct(
        public readonly int $priority = 0,
        array &$items,
    ) {}
}

final class VariadicByReference
{
    public function __construct(
        public readonly int $priority = 0,
        string &...$renders,
    ) {}
}

final class UntypedVariadic
{
    public function __construct(
        public readonly int $priority = 0,
        ...$renders,
    ) {}
}
