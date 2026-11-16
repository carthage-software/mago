<?php

declare(strict_types=1);

/** @return list<array<string, string>> */
function rows(): array
{
    return [
        ['k0' => 'v0'],
        ['k0' => 'v1'],
        ['k0' => 'v2'],
        ['k0' => 'v3'],
        ['k0' => 'v4'],
        ['k0' => 'v5'],
        ['k0' => 'v6'],
        ['k0' => 'v7'],
        ['k0' => 'v8'],
        ['k0' => 'v9'],
    ];
}

function f(): void
{
    $out1 = [];
    foreach (rows() as $row1) {
        $a1 = $row1['k0'] ?? null;
        if ($a1 === 'v0') {
            $out1[] = $a1;
            continue;
        }

        $out2 = [];
        foreach (rows() as $row2) {
            $a2 = $row2['k0'] ?? null;
            if ($a2 === 'v0') {
                $out2[] = $a2;
                continue;
            }

            $out3 = [];
            foreach (rows() as $row3) {
                $a3 = $row3['k0'] ?? null;
                if ($a3 === 'v0') {
                    $out3[] = $a3;
                    continue;
                }

                $out4 = [];
                foreach (rows() as $row4) {
                    $a4 = $row4['k0'] ?? null;
                    if ($a4 === 'v0') {
                        $out4[] = $a4;
                        continue;
                    }

                    $out5 = [];
                    foreach (rows() as $row5) {
                        $a5 = $row5['k0'] ?? null;
                        if ($a5 === 'v0') {
                            $out5[] = $a5;
                            continue;
                        }

                        $out6 = [];
                        foreach (rows() as $row6) {
                            $a6 = $row6['k0'] ?? null;
                            if ($a6 === 'v0') {
                                $out6[] = $a6;
                                continue;
                            }

                            $out7 = [];
                            foreach (rows() as $row7) {
                                $a7 = $row7['k0'] ?? null;
                                if ($a7 === 'v0') {
                                    $out7[] = $a7;
                                    continue;
                                }

                                $out8 = [];
                                foreach (rows() as $row8) {
                                    $a8 = $row8['k0'] ?? null;
                                    if ($a8 === 'v0') {
                                        $out8[] = $a8;
                                        continue;
                                    }

                                    $out9 = [];
                                    foreach (rows() as $row9) {
                                        $a9 = $row9['k0'] ?? null;
                                        if ($a9 === 'v0') {
                                            $out9[] = $a9;
                                            continue;
                                        }

                                        $out10 = [];
                                        foreach (rows() as $row10) {
                                            $a10 = $row10['k0'] ?? null;
                                            if ($a10 === 'v0') {
                                                $out10[] = $a10;
                                                continue;
                                            }

                                            $out11 = [];
                                            foreach (rows() as $row11) {
                                                $a11 = $row11['k0'] ?? null;
                                                if ($a11 === 'v0') {
                                                    $out11[] = $a11;
                                                    continue;
                                                }

                                                $out12 = [];
                                                foreach (rows() as $row12) {
                                                    $a12 = $row12['k0'] ?? null;
                                                    if ($a12 === 'v0') {
                                                        $out12[] = $a12;
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
