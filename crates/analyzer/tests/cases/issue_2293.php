<?php

declare(strict_types=1);

final class C
{
    public static string $p = '';
    public static ?string $nullable = null;
}

function sink(string $_): void {}

function repro(bool $flag): void
{
    if (C::$p !== '' && $flag) {
        sink(C::$p);
    }

    if (C::$nullable !== null && $flag) {
        sink(C::$nullable);
    }
}
