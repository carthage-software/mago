<?php

declare(strict_types=1);

/**
 * @param list<string> $rows
 */
function nested(array $rows, mixed $m1, mixed $m2, mixed $m3, mixed $m4, mixed $m5, mixed $m6, mixed $m7, mixed $m8, mixed $m9, mixed $m10, mixed $m11, mixed $m12): void
{
    $out1 = [];
    foreach ($rows as $row1) {
        if ($m1 === $row1) {
            $out1[] = $row1;
            continue;
        }

        $out2 = [];
        foreach ($rows as $row2) {
            if ($m2 === $row2) {
                $out2[] = $row2;
                continue;
            }

            $out3 = [];
            foreach ($rows as $row3) {
                if ($m3 === $row3) {
                    $out3[] = $row3;
                    continue;
                }

                $out4 = [];
                foreach ($rows as $row4) {
                    if ($m4 === $row4) {
                        $out4[] = $row4;
                        continue;
                    }

                    $out5 = [];
                    foreach ($rows as $row5) {
                        if ($m5 === $row5) {
                            $out5[] = $row5;
                            continue;
                        }

                        $out6 = [];
                        foreach ($rows as $row6) {
                            if ($m6 === $row6) {
                                $out6[] = $row6;
                                continue;
                            }

                            $out7 = [];
                            foreach ($rows as $row7) {
                                if ($m7 === $row7) {
                                    $out7[] = $row7;
                                    continue;
                                }

                                $out8 = [];
                                foreach ($rows as $row8) {
                                    if ($m8 === $row8) {
                                        $out8[] = $row8;
                                        continue;
                                    }

                                    $out9 = [];
                                    foreach ($rows as $row9) {
                                        if ($m9 === $row9) {
                                            $out9[] = $row9;
                                            continue;
                                        }

                                        $out10 = [];
                                        foreach ($rows as $row10) {
                                            if ($m10 === $row10) {
                                                $out10[] = $row10;
                                                continue;
                                            }

                                            $out11 = [];
                                            foreach ($rows as $row11) {
                                                if ($m11 === $row11) {
                                                    $out11[] = $row11;
                                                    continue;
                                                }

                                                $out12 = [];
                                                foreach ($rows as $row12) {
                                                    if ($m12 === $row12) {
                                                        $out12[] = $row12;
                                                        continue;
                                                    }

                                                    echo count($out12);
                                                }

                                                echo count($out11);
                                            }

                                            echo count($out10);
                                        }

                                        echo count($out9);
                                    }

                                    echo count($out8);
                                }

                                echo count($out7);
                            }

                            echo count($out6);
                        }

                        echo count($out5);
                    }

                    echo count($out4);
                }

                echo count($out3);
            }

            echo count($out2);
        }

        echo count($out1);
    }
}

/**
 * @param list<int> $items
 */
function untouched(array $items, mixed $value, bool $flag): void
{
    foreach ($items as $_) {
        if (!is_string($value) && !$flag) {
            continue;
        }

        foreach ($items as $item) {
            echo $item;
        }

        if (!$flag) {
            /** @mago-expect analysis:redundant-type-comparison */
            /** @mago-expect analysis:redundant-condition */
            if (is_string($value)) {
                echo 'string';
            }
        }
    }
}

/**
 * @param list<int> $items
 */
function written_through_reference(array $items, mixed $value, bool $flag): void
{
    foreach ($items as $_) {
        // @mago-expect analysis:mixed-assignment
        $alias = &$value;
        if (!is_string($value) && !$flag) {
            continue;
        }

        foreach ($items as $_) {
            $alias = null;
        }

        if (!$flag && is_string($value)) {
            echo 'string';
        }
    }
}

/**
 * @param list<int> $items
 */
function written_through_shared_reference(array $items, mixed $source, bool $flag): void
{
    foreach ($items as $_) {
        // @mago-expect analysis:mixed-assignment
        $value = &$source;
        // @mago-expect analysis:mixed-assignment
        $alias = &$source;
        if (!is_string($value) && !$flag) {
            continue;
        }

        foreach ($items as $_) {
            $alias = null;
        }

        if (!$flag && is_string($value)) {
            echo 'string';
        }
    }
}

/**
 * @param list<int> $items
 */
function reassigned(array $items, mixed $value, mixed $next, bool $flag): void
{
    foreach ($items as $_) {
        if (!is_string($value) && !$flag) {
            continue;
        }

        foreach ($items as $_) {
            // @mago-expect analysis:mixed-assignment
            $value = $next;
        }

        if (!$flag && is_string($value)) {
            echo 'string';
        }
    }
}

/**
 * @param list<int> $items
 */
function reassigned_in_outer_loop(array $items, mixed $value, mixed $next, bool $flag): void
{
    if (!is_string($value) && !$flag) {
        return;
    }

    foreach ($items as $_) {
        // @mago-expect analysis:mixed-assignment
        $value = $next;
    }

    if (!$flag && is_string($value)) {
        echo 'string';
    }
}

/**
 * @param list<int> $items
 */
function written_by_extract(array $items, mixed $value, bool $flag): void
{
    foreach ($items as $_) {
        if (!is_string($value) && !$flag) {
            continue;
        }

        foreach ($items as $_) {
            extract(['value' => null]);
        }

        if (!$flag && is_string($value)) {
            echo 'string';
        }
    }
}

function clear_global(): void
{
    $GLOBALS['global_value'] = null;
}

$global_flag = (bool) rand(0, 1);
foreach ([1, 2] as $_) {
    // @mago-expect analysis:mixed-assignment
    $global_value = $GLOBALS['global_value'] ?? null;
    if (!is_string($global_value) && !$global_flag) {
        continue;
    }

    foreach ([1, 2] as $_) {
        clear_global();
    }

    if (!$global_flag && is_string($global_value)) {
        echo 'string';
    }
}
