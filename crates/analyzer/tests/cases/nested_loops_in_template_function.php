<?php

declare(strict_types=1);

/**
 * @template T
 * @param list<string> $rows
 * @param T $t
 */
function nested_template(array $rows, mixed $t, mixed $m1, mixed $m2, mixed $m3, mixed $m4, mixed $m5, mixed $m6, mixed $m7, mixed $m8): void
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
 * @template T
 * @param list<string> $items
 * @param T $default
 * @return list<T>
 */
function inline_template_var(array $items, mixed $default): array
{
    $result = [$default];
    foreach ($items as $outer) {
        foreach ($items as $inner) {
            /** @var T $value */
            $value = $inner;
            $result[] = $value;
        }

        /** @var T $value */
        $value = $outer;
        $result[] = $value;
    }

    return $result;
}

/**
 * @template T
 */
final class Box
{
    /**
     * @param list<string> $items
     * @return list<T>
     */
    public function collect(array $items): array
    {
        $result = [];
        foreach ($items as $outer) {
            foreach ($items as $inner) {
                /** @var T $value */
                $value = $inner;
                $result[] = $value;
            }

            /** @var T $value */
            $value = $outer;
            $result[] = $value;
        }

        return $result;
    }
}

/**
 * @template T
 * @param list<string> $items
 * @param T $default
 * @return list<T>
 */
function closure_with_own_template(array $items, mixed $default): array
{
    $result = [$default];
    foreach ($items as $_) {
        $map =
            /**
             * @template U
             * @param U $seed
             * @return list<U>
             */
            static function (mixed $seed) use ($items): array {
                $seeds = [];
                foreach ($items as $outer) {
                    foreach ($items as $inner) {
                        /** @var U $value */
                        $value = $inner;
                        $seeds[] = $value;
                    }

                    /** @var U $value */
                    $value = $outer;
                    $seeds[] = $value;
                }

                $seeds[] = $seed;
                return $seeds;
            };

        foreach ($items as $_) {
            /** @var T $value */
            $value = $default;
            $result = [...$result, ...$map($value)];
        }
    }

    return $result;
}

/**
 * @template T of string
 * @param list<int> $items
 * @param T $value
 */
function template_widened_between_passes(array $items, string $value): void
{
    $current = $value;
    foreach ($items as $item) {
        foreach ($items as $_) {
            // @mago-expect analysis:possibly-invalid-argument
            echo strlen($current);
        }

        $current = $item;
    }
}

/**
 * @template T
 * @param list<int> $items
 * @param T $value
 */
function template_issue_in_inner_loop(array $items, mixed $value): void
{
    foreach ($items as $_) {
        foreach ($items as $_) {
            // @mago-expect analysis:less-specific-nested-argument-type
            echo strlen($value);
        }
    }
}

/**
 * @phpstan-type Row array{id: int}
 */
final class RowCollector
{
    /**
     * @param list<array<string, int>> $items
     * @return list<Row>
     */
    public function collect(array $items): array
    {
        $result = [];
        foreach ($items as $outer) {
            foreach ($items as $inner) {
                /** @var Row $row */
                $row = $inner;
                $result[] = $row;
            }

            /** @var Row $row */
            $row = $outer;
            $result[] = $row;
        }

        return $result;
    }

    /**
     * @param list<Row> $items
     */
    public function lengths(array $items): int
    {
        $total = 0;
        foreach ($items as $outer) {
            foreach ($items as $inner) {
                // @mago-expect analysis:invalid-argument
                $total += strlen($inner['id']) + $outer['id'];
            }
        }

        return $total;
    }
}
