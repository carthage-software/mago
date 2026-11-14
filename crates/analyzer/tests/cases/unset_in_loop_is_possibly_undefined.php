<?php

declare(strict_types=1);

final class LoopUnsetA
{
    public function noop(): void
    {
        echo 1;
    }
}

function loop_unset_in_foreach(): void
{
    $a = new LoopUnsetA();
    foreach ([1, 2] as $_) {
        if (isset($a)) {
            $a->noop();
        }

        unset($a);
    }
}

function loop_unset_in_while(): void
{
    $a = new LoopUnsetA();
    while (true) {
        if (!isset($a)) {
            return;
        }

        $a->noop();
        unset($a);
    }
}

/**
 * @param list<int> $xs
 */
function loop_unset_after_foreach(array $xs): void
{
    $a = new LoopUnsetA();
    foreach ($xs as $_) {
        unset($a);
    }

    // @mago-expect analysis:possibly-undefined-variable
    $a->noop();
}
