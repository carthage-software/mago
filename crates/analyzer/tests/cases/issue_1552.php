<?php

declare(strict_types=1);

/**
 * @param array<string,string> $meta
 */
function img_meta_get_lens(array $meta): string
{
    $a = ['LensID', 'LensSpec', 'Lens', 'LensModel', 'LensInfo'];

    $s = '';

    foreach ($a as $k) {
        if (isset($meta[$k])) {
            $t = trim($meta[$k]);
            $garbage = match (true) { true => false };
            continue;
        }
    }

    return $s;
}
