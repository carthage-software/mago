<?php

declare(strict_types=1);

final class Plain
{
    public int $x = 1;
}

function probe(): int
{
    return strlen(new Plain());
}
