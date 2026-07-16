<?php

declare(strict_types=1);

/**
 * @param positive-int $i
 */
function pos(int $i): void
{
    // @mago-expect analysis:redundant-comparison,redundant-condition
    if ($i > 0) {
    }
    // @mago-expect analysis:redundant-comparison,impossible-condition
    if ($i === -1) {
    }
    // @mago-expect analysis:redundant-comparison,redundant-condition
    if ($i >= 1) {
    }
    // @mago-expect analysis:redundant-comparison,redundant-condition
    if ($i !== -1) {
    }
    // @mago-expect analysis:redundant-comparison,redundant-condition
    if ($i !== -1) {
    }
}

/**
 * @param negative-int $i
 */
function neg(int $i): void
{
    // @mago-expect analysis:redundant-comparison,redundant-condition
    if ($i < 0) {
    }
    // @mago-expect analysis:redundant-comparison,impossible-condition
    if ($i === 1) {
    }
    // @mago-expect analysis:redundant-comparison,redundant-condition
    if ($i <= 1) {
    }
    // @mago-expect analysis:redundant-comparison,redundant-condition
    if ($i !== 1) {
    }
    // @mago-expect analysis:redundant-comparison,redundant-condition
    if ($i !== 1) {
    }
}

/** @param non-negative-int $i */
function maybe_positive(int $i): void
{
    if ($i > 0) {
        echo 'positive';
    }
}

/** @param non-positive-int $i */
function maybe_negative(int $i): void
{
    if ($i < 0) {
        echo 'negative';
    }
}
