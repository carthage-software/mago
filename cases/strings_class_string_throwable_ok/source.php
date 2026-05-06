<?php

declare(strict_types=1);

/** @param class-string<Throwable> $c */
function takes_throwable_cs(string $c): void
{
    echo $c;
}

takes_throwable_cs(Exception::class);
takes_throwable_cs(\RuntimeException::class);
