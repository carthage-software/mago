<?php

declare(strict_types=1);

namespace ArrayShapeDiscriminantMatch;

/**
 * @param array{type: 'Illust', image_url: non-falsy-string}|array{type: 'Novel', cover_url: non-falsy-string} $work
 */
function coverUrl(array $work): string
{
    return match ($work['type']) {
        'Illust' => $work['image_url'],
        'Novel' => $work['cover_url'],
    };
}

/**
 * @param array{type: 'Illust', image_url: non-falsy-string}|array{type: 'Novel', cover_url: non-falsy-string} $work
 */
function crossedArms(array $work): string
{
    // @mago-expect analysis:invalid-return-statement
    return match ($work['type']) {
        // @mago-expect analysis:undefined-string-array-index
        'Illust' => $work['cover_url'],
        // @mago-expect analysis:undefined-string-array-index
        'Novel' => $work['image_url'],
    };
}

/**
 * @param array{type: 'Illust', image_url: non-falsy-string}|array{type: 'Novel', cover_url: non-falsy-string} $work
 */
function coverUrlWithIf(array $work): string
{
    if ($work['type'] === 'Illust') {
        return $work['image_url'];
    }

    return $work['cover_url'];
}
