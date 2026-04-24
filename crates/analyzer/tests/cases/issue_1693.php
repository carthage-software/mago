<?php

declare(strict_types=1);

final class Graph
{
    private const int LABEL_WHITE = 0;
    private const int LABEL_GRAY = 1;
    private const int LABEL_BLACK = 2;

    /**
     * @param list<int> $nodes
     * @param array<int, list<int>> $graph
     */
    public static function hasCycle(array $nodes, array $graph): bool
    {
        $labels = [];

        foreach ($nodes as $node) {
            $labels[$node] = self::LABEL_WHITE;
        }

        foreach ($nodes as $node) {
            if (
                $labels[$node] === self::LABEL_WHITE
                && isset($graph[$node])
                && static::dfsCheckCycles($graph, $node, $labels)
            ) {
                return true;
            }
        }

        return false;
    }

    /**
     * @param array<int, list<int>> $graph
     * @param array<int, self::LABEL_*> $labels
     */
    private static function dfsCheckCycles(
        array $graph,
        int $start,
        array &$labels,
    ): bool {
        $stack = [$start];

        while ($stack !== []) {
            $node = \end($stack);

            if ($labels[$node] === self::LABEL_WHITE) {
                $labels[$node] = self::LABEL_GRAY;

                foreach ($graph[$node] as $edge) {
                    if (
                        $labels[$edge] === self::LABEL_WHITE
                        && isset($graph[$edge])
                    ) {
                        $stack[] = $edge;
                    } elseif ($labels[$edge] === self::LABEL_GRAY) {
                        return true;
                    }
                }
            } elseif ($labels[$node] === self::LABEL_GRAY) {
                \array_pop($stack);
                $labels[$node] = self::LABEL_BLACK;
            } else {
                \array_pop($stack);
            }
        }

        return false;
    }
}
