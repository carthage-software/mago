<?php

declare(strict_types=1);

/**
 * @param list<string> $words
 */
function nullable_counter(array $words): void
{
    $count = null;
    foreach ($words as $_) {
        $count = ($count ?? 0) + 1;
    }

    if ($count === 5) {
        echo 'five words';
    }
}

/**
 * @param list<string> $words
 */
function word_counts(array $words): void
{
    $counts = [];
    foreach ($words as $word) {
        $counts[$word] = ($counts[$word] ?? 0) + 1;
    }

    foreach ($counts as $word => $count) {
        if ($count > 3) {
            echo $word;
        }
    }
}

function literal_union_counter(bool $odd): void
{
    $total = $odd ? 1 : 0;
    foreach ([1, 2, 3, 4, 5, 6] as $_) {
        $total += 2;
    }

    if ($total === 12) {
        echo 'twelve';
    }
}
