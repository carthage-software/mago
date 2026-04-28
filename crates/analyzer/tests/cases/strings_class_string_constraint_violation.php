<?php

declare(strict_types=1);

/** @param class-string<Throwable> $c */
function takes_throwable_cs(string $c): void
{
    echo $c;
}

final class NotAThrowable {}

/** @mago-expect analysis:invalid-argument */
takes_throwable_cs(NotAThrowable::class);
