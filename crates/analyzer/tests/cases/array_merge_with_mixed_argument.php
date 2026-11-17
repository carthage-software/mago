<?php

declare(strict_types=1);

function array_merge_with_mixed(mixed $overrides): void
{
    $defaults = ['term' => null, 'enabled' => false];
    /** @mago-expect analysis:mixed-argument */
    $options = array_merge($defaults, $overrides);

    if (!empty($options['term']) && !is_array($options['term'])) {
        echo 'scalar term';
    }
}
