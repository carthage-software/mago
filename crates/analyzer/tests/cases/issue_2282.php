<?php

declare(strict_types=1);

function take(\Closure $closure): void
{
}

/** @param non-empty-array<string, \Closure|list<\Closure>> $steps */
function viaIsArray(array $steps): void
{
    foreach ($steps as $stepData) {
        if (!\is_array($stepData)) {
            $stepData = [$stepData];
        }

        foreach ($stepData as $closure) {
            take($closure);
        }
    }
}

/** @param non-empty-array<string, \Closure|list<\Closure>> $steps */
function viaInstanceof(array $steps): void
{
    foreach ($steps as $stepData) {
        if ($stepData instanceof \Closure) {
            $stepData = [$stepData];
        }

        foreach ($stepData as $closure) {
            take($closure);
        }
    }
}
