<?php

declare(strict_types=1);

function pick(?string $s): string
{
    while (true) {
        switch (rand(0, 1)) {
            case 0:
                $s ??= 'x';
                break 2;
            default:
                break;
        }
    }

    return $s;
}

/**
 * @return 0
 */
function settle(int $mode): int
{
    while (true) {
        switch ($mode) {
            case 1:
                break;
            case 0:
                break 2;
        }

        $mode = rand(0, 1);
    }

    return $mode;
}

/**
 * @param list<int> $items
 */
function find_positive(array $items): int
{
    while (true) {
        switch (rand(0, 1)) {
            case 0:
                foreach ($items as $item) {
                    if ($item > 0) {
                        $found = $item;
                        break 2;
                    }
                }

                break;
            default:
                return 0;
        }

        if (isset($found)) {
            return $found;
        }
    }
}
