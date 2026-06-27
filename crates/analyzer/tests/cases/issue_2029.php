<?php

declare(strict_types=1);

function use_item(int $item): void {}

function drain(): void
{
    $queue = [1];
    do {
        $item = array_shift($queue);
        use_item($item);
    } while ($queue);
}

final class Node
{
    public int $value = 0;
}

function next_node(): ?Node
{
    return null;
}

function walk(): void
{
    $node = next_node();
    do {
        /** @mago-expect analysis:possibly-null-property-access */
        echo $node->value;
    } while ($node = next_node());
}

function walk2(): void
{
    $node = next_node();
    if (null === $node) {
        return;
    }

    do {
        echo $node->value;
    } while ($node = next_node());
}
