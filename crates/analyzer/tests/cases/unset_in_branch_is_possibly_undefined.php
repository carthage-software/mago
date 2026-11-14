<?php

declare(strict_types=1);

final class BranchUnsetA
{
    public function noop(): void
    {
        echo 1;
    }
}

function call_after_branch_unset(bool $b): void
{
    $a = new BranchUnsetA();
    if ($b) {
        unset($a);
    }

    // @mago-expect analysis:possibly-undefined-variable
    $a->noop();
}

function isset_after_branch_unset(bool $b): void
{
    $a = new BranchUnsetA();
    if ($b) {
        unset($a);
    }

    if (isset($a)) {
        $a->noop();
    }
}

function coalesce_after_branch_unset(bool $b): ?BranchUnsetA
{
    $a = new BranchUnsetA();
    if ($b) {
        unset($a);
    }

    return $a ?? null;
}
