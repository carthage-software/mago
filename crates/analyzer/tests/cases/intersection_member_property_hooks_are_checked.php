<?php

declare(strict_types=1);

interface A
{
}

interface B
{
    public bool $bar { get; }
}

interface C
{
    public bool $baz { set; }
}

function write_get_only(A&B $a): void
{
    // @mago-expect analysis:invalid-property-write
    $a->bar = true;
}

function write_get_only_narrowed(A $a): void
{
    if ($a instanceof B) {
        // @mago-expect analysis:invalid-property-write
        $a->bar = true;
    }
}

function read_set_only(A&C $c): void
{
    // @mago-expect analysis:invalid-property-read
    echo $c->baz ? 'y' : 'n';
}

function read_get_only(A&B $a): bool
{
    return $a->bar;
}
