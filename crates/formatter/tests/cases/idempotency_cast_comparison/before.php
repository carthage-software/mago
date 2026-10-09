<?php

function f(): bool
{
    return 1 === (int) g(
        $a,
    );
}

function method(): bool
{
    return $a !== (int) $this->c->fetchOne(
        $query,
    );
}

$a < (float) g(
    $b,
);

1 >= (int) (float) g(
    $a,
);

function preserved(): bool
{
    return 1
        === (int) g(
            $a,
        );
}
