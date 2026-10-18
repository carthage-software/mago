<?php

declare(strict_types=1);

function acceptString(string $_): void {}

function analyzeEmptyLoopBody(): void
{
    // @mago-expect analysis:mixed-assignment
    foreach ([] as $value) {
        // @mago-expect analysis:mixed-argument
        acceptString($value);
    }
}
