<?php

declare(strict_types=1);

class GrandParentClass
{
}

class ParentClass extends GrandParentClass
{
    /** @var parent|null */
    public $up;
}

final class ChildClass extends ParentClass
{
    public $up;
}

function assign_grand_parent(ChildClass $c): void
{
    $c->up = new GrandParentClass();
}

trait Linked
{
    /** @var self|null */
    public $next;
}

class Node
{
    use Linked;

    public function hello(): string
    {
        return 'hi';
    }
}

final class LeafNode extends Node
{
    public $next;
}

function next_greeting(LeafNode $node): ?string
{
    return $node->next?->hello();
}
