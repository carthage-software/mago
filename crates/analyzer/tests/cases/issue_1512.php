<?php

declare(strict_types=1);

function nanothumb_sprint(int $x): string
{
    return "n{$x}";
}

/**
 * @return false|array<'element_id',int>
 */
function FASSOC(): false|array
{
    if (mt_rand(0, max: 10)) {
        return ['element_id' => mt_rand(1, max: 10000)];
    }
    return false;
}

function ser_event_handler_ser_snippet_details(string &$out): int
{
    $ia = [];
    $id = 0;
    $last = 0;
    while ($row = FASSOC()) {
        $last = $id;
        $id = $row['element_id'];
        if (count($ia) < 2) {
            $ia[] = $id;
        }
    }
    if ($last && !in_array($last, $ia, strict: true)) {
        $ia[2] = $last;
    }
    if ($id && !in_array($id, $ia, strict: true)) {
        $ia[3] = $id;
    }
    if (count($ia)) {
        $out .= empty($ia[0]) ? '' : nanothumb_sprint($ia[0]);
        $out .= empty($ia[1]) ? '' : nanothumb_sprint($ia[1]);
        $out .= empty($ia[2]) ? '' : nanothumb_sprint($ia[2]);
        $out .= empty($ia[3]) ? '' : nanothumb_sprint($ia[3]);
    }

    return 1;
}
