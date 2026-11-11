<?php

$this->getResult()
    ->items
    ->count();

$builder
    ->method('getResults')
    ->with(1)->willReturn([1]);

$a
    ->b()->c();

$builder
    ->a()
    ->b()->c()->d();

$this?->getResult()
    ?->items
    ?->count();

$query->getResult()
    ->items
    // Count the items.
    ->count();
