<?php

declare(strict_types=1);

enum TaskState: int
{
    case Queued = 1;
}

interface TaskInterfaceA
{
    public const int STATE_QUEUED = 1;
}

interface TaskInterfaceB
{
    public const int STATE_QUEUED = TaskState::Queued->value;
}

/**
 * @param 1 $a
 */
function delta(int $a): void
{
    echo $a;
}

/** @param TaskInterfaceA::STATE_* $stateA */
function a(int $stateA): void
{
    delta($stateA);
}

/** @param TaskInterfaceB::STATE_* $stateB */
function b(int $stateB): void
{
    delta($stateB);
}
