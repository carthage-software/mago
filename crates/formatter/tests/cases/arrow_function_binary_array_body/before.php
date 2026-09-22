<?php

usort($records, static fn (array $a, array $b): int => [
    $a['x'],
] <=> [
    $b['x'],
]);
