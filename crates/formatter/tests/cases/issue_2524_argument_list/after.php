<?php

$url =
    config('app.url')
    . route(
        'listings.index',
        [
            'listing_id' => $listing->id,
        ],
        false,
    );

$url =
    config('app.url')
    . route('listings.index', ['listing_id' => $listing->id, 'other_param' => $other->value, 'x' => 1], false);

$listing = $this->repository->find(
    $id,
    $other,
) ?? [];

final class Routes
{
    const INDEX =
        self::PREFIX
            . sprintf('%s %s %s', $aaaaaaaaaaaaaaaaaaaa, $bbbbbbbbbbbbbbbbbbbbbbbb, $cccccccccccc, $dddddddddddd);

    public $show =
        self::PREFIX . sprintf('%s %s %s', $aaaaaaaaaaaaaaaaaaaa, $bbbbbbbbbbbbbbbbbbbbbbbb, $cccccccccccc, $ddddddddd);
}

$routes = [
    'index' =>
        self::PREFIX
            . sprintf('%s %s %s', $aaaaaaaaaaaaaaaaaaaa, $bbbbbbbbbbbbbbbbbbbbbbbb, $cccccccccccc, $ddddddddddddd),
];
