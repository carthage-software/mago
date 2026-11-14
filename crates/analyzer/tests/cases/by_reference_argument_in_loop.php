<?php

declare(strict_types=1);

/**
 * @param list<string> $subjects
 */
function by_ref_in_loop_firsts(array $subjects): void
{
    foreach ($subjects as $subject) {
        if (preg_match('/a+/', $subject, $matches)) {
            echo $matches[0] ?? '';
        }
    }
}

function by_ref_in_loop_fill(?int &$out): void
{
    $out = 1;
}

/**
 * @param list<string> $subjects
 */
function by_ref_in_loop_fill_each(array $subjects): void
{
    foreach ($subjects as $_) {
        by_ref_in_loop_fill($value);
        echo $value;
    }
}
