<?php

declare(strict_types=1);

final class Container
{
    public null|string $value = null;

    public function maybeChange(): void
    {
    }
}

function flow_narrow_lost_after_call(Container $c): int
{
    if ($c->value === null) {
        return 0;
    }

    $len = strlen($c->value);

    $c->maybeChange();

    return $len;
}
