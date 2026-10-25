<?php

declare(strict_types=1);

namespace Issue2619;

function takesNull(null $value): void {}

function takesInt(int $value): void {}

function elseReturns(?int $a, ?int $b): void
{
    if ($a !== null) {
    } elseif ($b !== null) {
    } else {
        return;
    }

    if ($b === null) {
        takesNull($b);
    }

    // @mago-expect analysis:possibly-null-argument
    takesInt($b);
}

function elseAssigns(?int $a, ?int $b): void
{
    if ($a !== null) {
    } elseif ($b !== null) {
    } else {
        $b = 1;
    }

    if ($b === null) {
        takesNull($b);
    }

    // @mago-expect analysis:possibly-null-argument
    takesInt($b);
}

function correlatedBranches(?string $a, ?string $b): string
{
    if ($a) {
    } elseif ($b) {
    } else {
        return 'neither';
    }

    if (!$a) {
        return $b;
    }

    return $a;
}

function threeCorrelatedBranches(?string $a, ?string $b, ?string $c): string
{
    if ($a) {
    } elseif ($b) {
    } elseif ($c) {
    } else {
        return 'neither';
    }

    if (!$a && !$b) {
        return $c;
    }

    if (!$a) {
        return $b;
    }

    return $a;
}

function allBranchesNarrow(?int $a, ?int $b): int
{
    if ($a !== null) {
        $b = $a;
    } elseif ($b !== null) {
    } else {
        return 0;
    }

    return $b;
}

function earlierBranchAssigns(?int $a, ?int $b): void
{
    if ($a !== null) {
        $b = null;
    } elseif ($b !== null) {
    } else {
        return;
    }

    if ($b === null) {
        takesNull($b);
    }

    // @mago-expect analysis:possibly-null-argument
    takesInt($b);
}

function multipleElseifs(?int $a, ?int $b, ?int $c): void
{
    if ($a !== null) {
    } elseif ($b !== null) {
    } elseif ($c !== null) {
    } else {
        return;
    }

    if ($b === null) {
        takesNull($b);
    }

    if ($c === null) {
        takesNull($c);
    }

    // @mago-expect analysis:possibly-null-argument
    takesInt($b);
    // @mago-expect analysis:possibly-null-argument
    takesInt($c);
}

function onlyElseifContinues(?int $a, ?int $b): int
{
    if ($a !== null) {
        return $a;
    } elseif ($b !== null) {
    } else {
        return 0;
    }

    return $b;
}

function elseifReassigns(?int $a, ?int $b): void
{
    if ($a !== null) {
    } elseif ($b !== null) {
        $b = null;
    } else {
        return;
    }

    if ($a === null) {
        // @mago-expect analysis:possibly-null-argument
        takesInt($b);
    }
}

function firstBranchReassigns(?int $a, ?int $b): void
{
    if ($a !== null) {
        $a = null;
    } elseif ($b !== null) {
    } else {
        return;
    }

    if ($b === null) {
        takesNull($b);
    }

    // @mago-expect analysis:possibly-null-argument
    takesInt($b);
}

final class Box
{
    public function __construct(
        public ?int $value,
    ) {}

    public function clear(): void
    {
        $this->value = null;
    }
}

function clearBox(Box $box): void
{
    $box->value = null;
}

function elseifMutatesProperty(?int $a, Box $box): void
{
    if ($a !== null) {
    } elseif ($box->value !== null) {
        clearBox($box);
    } else {
        return;
    }

    if ($a === null) {
        // @mago-expect analysis:possibly-null-argument
        takesInt($box->value);
    }
}

function elseifCallsMethod(?int $a, Box $box): void
{
    if ($a !== null) {
    } elseif ($box->value !== null) {
        $box->clear();
    } else {
        return;
    }

    if ($a === null) {
        // @mago-expect analysis:possibly-null-argument
        takesInt($box->value);
    }
}

function elseContinues(?int $a, ?int $b): void
{
    foreach ([1, 2] as $_) {
        if ($a !== null) {
        } elseif ($b !== null) {
        } else {
            continue;
        }

        if ($b === null) {
            takesNull($b);
        }

        // @mago-expect analysis:possibly-null-argument
        takesInt($b);
    }
}
