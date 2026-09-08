<?php

declare(strict_types=1);

function takesInteger(int $_): void {}

function takesFalse(false $_): void {}

function compareStrposResult(string $subject): bool
{
    return strpos($subject, 'x') > 0;
}

function comparePositiveInteger(int|false $result): void
{
    if ($result > 0) {
        takesInteger($result);
    } elseif ($result === false) {
        takesFalse($result);
    }
}

function comparePositiveIntegerReversed(int|false $result): void
{
    if (0 < $result) {
        takesInteger($result);
    } elseif ($result === false) {
        takesFalse($result);
    }
}

function compareOtherThresholds(int|false $result, int $threshold): void
{
    // @mago-expect analysis:possibly-false-operand
    if ($result >= 0) {
    }

    // @mago-expect analysis:possibly-false-operand
    if ($result == 0) {
    }

    // @mago-expect analysis:possibly-false-operand
    if ($result > $threshold) {
    }

    // @mago-expect analysis:possibly-false-operand
    if ($result > -1) {
    }
}

function compareOtherTypes(float|false $float, string|false $string, int|false|null $nullable): void
{
    // @mago-expect analysis:possibly-false-operand
    if ($float > 0) {
    }

    // @mago-expect analysis:possibly-false-operand
    if ($string > 0) {
    }

    // @mago-expect analysis:possibly-null-operand
    if ($nullable > 0) {
    }
}

/** @param int<-1, 0> $threshold */
function compareRangedThreshold(int|false $result, int $threshold): void
{
    // @mago-expect analysis:possibly-false-operand
    if ($result < $threshold) {
        if ($result === false) {
            takesFalse($result);
        }
    }
}

function compareDefiniteFalse(false $result): bool
{
    // @mago-expect analysis:false-operand,redundant-comparison
    return $result > 0;
}
