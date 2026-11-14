<?php

declare(strict_types=1);

function fresh(): ?string
{
    return rand(0, 1) ? 'x' : null;
}

function while_loop(?string $s): int
{
    while (null === $s && null !== ($s = fresh())) {
        return strlen($s);
    }

    return 0;
}

function for_loop(?string $s): int
{
    for (; null === $s && null !== ($s = fresh());) {
        return strlen($s);
    }

    return 0;
}
