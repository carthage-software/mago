<?php

declare(strict_types=1);

final class Foo
{
    public function hi(): string
    {
        return 'hi';
    }
}

class Bar
{
    public function count(): int
    {
        return 1;
    }
}

function by_final_class(object $o): string
{
    return match (get_class($o)) {
        Foo::class => $o->hi(),
        default => '',
    };
}

function by_open_class(object $o): int
{
    return match (get_class($o)) {
        Bar::class => $o->count(),
        default => 0,
    };
}

function by_type(int|string $value): string
{
    return match (gettype($value)) {
        'integer' => (string) ($value + 1),
        default => $value,
    };
}

function by_several_classes(Foo|Bar $o): string
{
    return match (get_class($o)) {
        Foo::class => $o->hi(),
        default => (string) $o->count(),
    };
}
