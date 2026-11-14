<?php

declare(strict_types=1);

final class Bar
{
    public function touch(): void {}
}

final class Foo
{
    public function __construct(
        public Bar $bar,
    ) {}
}

function stored(?Foo $foo): void
{
    $bar = $foo?->bar;

    // @mago-expect analysis:possible-method-access-on-null
    $bar->touch();
}

function chained(?Foo $foo): void
{
    $foo?->bar->touch();
}
