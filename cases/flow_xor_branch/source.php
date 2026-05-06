<?php

declare(strict_types=1);

function flow_xor_branch(bool $a, bool $b): int
{
    if ($a xor $b) {
        return 1;
    }

    return 0;
}
