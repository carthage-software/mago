<?php

declare(strict_types=1);

final class A
{
    public function __construct(
        public readonly string $a,
    ) {}
}

final class B
{
    public function __construct(
        public readonly string $b,
    ) {}
}

final class C
{
    public function __get(string $name): string
    {
        return $name;
    }
}

function coalesce_on_union(A|B $obj): string
{
    return $obj->a ?? '';
}

function isset_on_union(B|A $obj): bool
{
    return isset($obj->a);
}

function isset_on_magic_get(C $x): string
{
    // @mago-expect analysis:non-documented-property
    return isset($x->a) ? 'set' : 'unset';
}

function coalesce_on_magic_get(C $x): string
{
    // @mago-expect analysis:non-documented-property,redundant-null-coalesce
    return $x->a ?? 'none';
}
