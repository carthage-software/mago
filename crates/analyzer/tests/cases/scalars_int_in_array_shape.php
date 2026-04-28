<?php

declare(strict_types=1);

/**
 * @param array{x: int, y: int} $point
 */
function dist_squared(array $point): int {
    return $point['x'] * $point['x'] + $point['y'] * $point['y'];
}

dist_squared(['x' => 3, 'y' => 4]);
