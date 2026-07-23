<?php

declare(strict_types=1);

/**
 * @return list<int>
 */
function issue2124Loop(): array
{
    $nums = [1, 2];

    try {
        while (true) {
            if (rand(0, 10) === 0) {
                break;
            }

            foreach ($nums as $num) {
            }
        }
    } catch (\Exception) {
        throw new \RuntimeException();
    }

    return $nums;
}

function issue2124NestedLoop(): array
{
    $nums = [1, 2];

    try {
        while (true) {
            foreach ($nums as $num) {
                if (rand(0, 10) === 0) {
                    break 2;
                }
            }
        }
    } catch (\Exception) {
        throw new \RuntimeException();
    }

    return $nums;
}
