<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:possibly-undefined-int-array-index(4)
 */
function issue_1668_table(): void
{
    $ia = [];
    $id = 0;
    $last = 0;
    while ($val = mt_rand(0, max: 20)) {
        $last = $id;
        $id = $val;
        if (count($ia) < 2) {
            $ia[] = $id;
        }
    }

    if ($last && !in_array($last, $ia, true)) {
        $ia[2] = $last;
    }

    if ($id && !in_array($id, $ia, true)) {
        $ia[3] = $id;
    }

    if (count($ia)) {
        $code = '<table>';
        $code .= '<tr>';
        $code .= '<td>' . ($ia[0] ? $ia[0] : '--');
        $code .= '<td>' . ($ia[1] ? $ia[1] : '--');
        $code .= '<tr>';
        $code .= '<td>' . ($ia[2] ? $ia[2] : '--');
        $code .= '<td>' . ($ia[3] ? $ia[3] : '--');
        $code .= '</table>';
        print $code;
    }
}
