<?php

declare(strict_types=1);

function bad(int $sep): array
{
    return explode($sep, 'a,b,c');
}
