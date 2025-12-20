<?php

declare(strict_types=1);

function returns_void(): void
{
}

if (returns_void()) { // @mago-expect analysis:impossible-condition
    echo 1;
}
