<?php

declare(strict_types=1);

trait Linked
{
    public ?self $next = null;
}

class Node
{
    use Linked;

    public function id(): int
    {
        return 1;
    }
}

final class SpecialNode extends Node {}

final class Other
{
    use Linked;
}

function next_id(Node $node): int
{
    return $node->next?->id() ?? 0;
}

function next_from_child(SpecialNode $node): int
{
    return $node->next?->id() ?? 0;
}

function link_nodes(Node $node, Other $other): void
{
    // @mago-expect analysis:invalid-property-assignment-value
    $node->next = $other;
}
