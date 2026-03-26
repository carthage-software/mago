<?php

$players = [];

$uniqueTeams = array_reduce(
    $players,
    static function (array $carry, string $player): array {
        return $carry + ['zxc' => $player]; // possibly-null-operand
    },
    [],
);
