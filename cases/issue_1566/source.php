<?php

declare(strict_types=1);

/** @return array<string,mixed> */
function setup_load_stanza(string $_string): array
{
    return [];
}

function sitemap_create_late(): ?string
{
    $param = [];
    $param['active'] = 0;
    $setup = setup_load_stanza('sitemap');

    if (isset($setup['param'])) {
        foreach ((array) $setup['param'] as $k => $v) {
            $param[(string) $k] = $v;
        }
    }

    if (!$param['active']) {
        return null;
    }

    if (isset($param['types_in'])) {
        $tin = implode(',', (array) $param['types_in']);
    }

    return 'done';
}
