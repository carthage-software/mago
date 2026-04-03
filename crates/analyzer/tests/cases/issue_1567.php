<?php

declare(strict_types=1);

/**
 * @param list<list<int>> $w
 */
function test(array $w): void
{
    $i = 0;
    $some =
        function () use ($w, &$i): ?array {
            if (0 === count($w) && 0 === $i) {
                ++$i;

                return [-1];
            }
            if ($i >= count($w)) {
                return null;
            }
            $d = $w[$i];
            ++$i;

            return $d;
        };

    $some();
    $some();
}
