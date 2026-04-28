<?php

declare(strict_types=1);

function bad(int $sep): array
{
    // @mago-expect analysis:invalid-argument
    return explode($sep, 'a,b,c');
}
