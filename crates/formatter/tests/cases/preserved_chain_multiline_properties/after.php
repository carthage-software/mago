<?php

$query->getResult()
    ->items->filter(function ($item) {
        return $item->active;
    });

$builder->method('getResults')
    ->prop->willReturn([
        ['id' => 1],
    ]);

$builder->method('getResults')
    ->willReturn([
        ['id' => 1],
    ])->prop;

$query?->getResult()
    ?->items?->filter(function ($item) {
        return $item?->active;
    });

$query->result->items->filter(function ($item) {
    return $item->active;
});

$query->getResult()
    // Filter the items.
    ->items->filter(function ($item) {
        return $item->active;
    });

expect(config('filesystems.disks.' . Disk::BASE_NAME))->toBeArray()->host->not->toBeNull();
