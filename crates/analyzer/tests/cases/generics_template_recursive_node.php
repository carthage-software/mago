<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenLinkedNode
{
    /**
     * @param T $value
     * @param GenLinkedNode<T>|null $next
     */
    public function __construct(public mixed $value, public ?GenLinkedNode $next = null)
    {
    }
}

$head = new GenLinkedNode(1, new GenLinkedNode(2, new GenLinkedNode(3)));
$cur = $head;
while (null !== $cur) {
    echo $cur->value;
    $cur = $cur->next;
}
