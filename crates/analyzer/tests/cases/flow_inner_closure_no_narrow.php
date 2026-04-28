<?php

declare(strict_types=1);

function flow_inner_closure_no_narrow(null|string $v): Closure
{
    if ($v === null) {
        $v = 'default';
    }

    return static function () use ($v): string {
        return $v;
    };
}
