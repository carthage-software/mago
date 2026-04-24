<?php

declare(strict_types=1);

/** @return false|array<string,mixed> */
function setup_load_stanza(string $_f): false|array
{
    if (!mt_rand(0, max: 100)) {
        return false;
    }

    return [];
}

function test(): bool
{
    $param = [];
    $param['active'] = 0;
    $setup = setup_load_stanza('sitemap');
    if (!is_array($setup)) {
        return false;
    }

    if (array_key_exists('param', $setup) && is_array($setup['param'])) {
        // @mago-expect analysis:mixed-assignment
        foreach ($setup['param'] as $k => $v) {
            $param[(string) $k] = $v;
        }

        unset($setup['param']);
    }

    if (!$param['active']) {
        return false;
    }

    return true;
}
