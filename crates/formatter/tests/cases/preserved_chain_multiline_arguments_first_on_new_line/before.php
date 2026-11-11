<?php

$builder->method('getResults')->willReturn([
    ['id' => 1],
]);

$builder?->method('getResults')?->willReturn([
    ['id' => 1],
]);

Builder::method('getResults')->willReturn([
    ['id' => 1],
]);
