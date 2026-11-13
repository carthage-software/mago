<?php

$builder->method('getResults')
    ->willReturn([
        ['id' => 1],
    ]);

$builder->method('getResults')
    ->willReturn([
        ['id' => 1],
    ]);

$mock->expects($this->once())
    ->method('find')
    ->with($id)
    ->willReturn([
        ['id' => 1],
    ]);
