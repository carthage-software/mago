<?php

declare(strict_types=1);

$items = [
    ['group' => 'A', 'name' => 'foo'],
    ['group' => 'A', 'name' => 'bar'],
];

$grouped = [];
foreach ($items as $item) {
    $grouped[$item['group']][] = $item['name'];
}

// mago incorrectly reports $grouped['A'] as type `never`
echo implode(', ', $grouped['A']); // outputs: foo, bar
