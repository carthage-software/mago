<?php

function shouldRetry(int $i, int $retries, int $offset, int $limit): bool
{
    if ($i === $retries - 1) {
        return false;
    }

    if ($offset !== $limit + 1) {
        return true;
    }

    return $limit <= $retries * 2;
}

$value = $maybe ?? $fallback + 1;
$result = $enabled ? $retries - 1 : $retries + 1;
$total = $now - (($i + 1) * $step);
$required = ($left === $right) - 1;

$comparisonWithoutGrouping = $i === $retries - 1;
$comparisonWithGrouping = $i === $retries - 1;
$coalesceWithoutGrouping = $maybe ?? $fallback + 1;
$coalesceWithGrouping = $maybe ?? $fallback + 1;
$requiredArithmeticGrouping = $now - (($i + 1) * $step);
$requiredComparisonGrouping = ($left === $right) - 1;
