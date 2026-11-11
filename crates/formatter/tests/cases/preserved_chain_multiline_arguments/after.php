<?php

$builder->method('getResults')
    ->willReturn([
        ['id' => 1],
    ]);

$builder?->method('getResults')
    ?->willReturn([
        ['id' => 1],
    ]);

Builder::method('getResults')->willReturn([
    ['id' => 1],
]);

$builder->method('getResults')
    ->willReturn([
        ['id' => 1],
    ]);

$builder
    // Select the method.
    ->method('getResults')
    ->willReturn([
        ['id' => 1],
    ]);

$builder->method('getResults')
    // Supply the results.
    ->willReturn([
        ['id' => 1],
    ]);

$builder
    ->willReturn([
        ['id' => 1],
    ]);
