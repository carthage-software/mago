<?php

declare(strict_types=1);

/**
 * @phpstan-type Tree array{value: int, children: array<int, Tree>}
 */
final class TreeBuilderBL
{
    /**
     * @return Tree
     */
    public function leaf(int $v): array
    {
        return ['value' => $v, 'children' => []];
    }

    /**
     * @param Tree $left
     * @param Tree $right
     *
     * @return Tree
     */
    public function branch(int $v, array $left, array $right): array
    {
        return ['value' => $v, 'children' => [$left, $right]];
    }
}

$b = new TreeBuilderBL();
$leaf = $b->leaf(1);
echo $leaf['value'];
