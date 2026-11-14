<?php

declare(strict_types=1);

function declared(mixed $m): ?int
{
    $g = function (mixed $v): ?int {
        /** @mago-expect analysis:mixed-return-statement */
        return $v;
    };

    return $g($m);
}

function arrow(mixed $m): ?int
{
    /** @mago-expect analysis:mixed-return-statement */
    $g = fn(mixed $v): ?int => $v;

    return $g($m);
}

/** @return list{int} */
function narrower_body(): array
{
    $g = static function (): array {
        return [1];
    };

    return $g();
}
