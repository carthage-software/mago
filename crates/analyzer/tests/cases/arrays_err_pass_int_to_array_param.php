<?php

declare(strict_types=1);

/** @param list<int> $xs */
function takes(array $xs): void
{
    foreach ($xs as $x) {
        echo $x;
    }
}

function call_bad(): void
{
    // @mago-expect analysis:invalid-argument
    takes(42);
}
