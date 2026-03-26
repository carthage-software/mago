<?php

declare(strict_types=1);

/**
 * @template TParent of object|null = null
 */
final class ChildNode
{
    /**
     * @param TParent $parent
     */
    public function __construct(
        private readonly ?object $parent = null,
    ) {}

    /**
     * @return $this
     */
    public function doChildThing(): static
    {
        echo 'doing child thing';

        return $this;
    }

    /**
     * @return TParent
     */
    public function end(): ?object
    {
        return $this->parent;
    }
}

final class ParentNode
{
    /**
     * @return ChildNode<$this>
     */
    public function thisChild(): ChildNode
    {
		return $this->thisChild();
    }

    /**
     * @return ChildNode<static>
     */
    public function staticChild(): ChildNode
    {
		return $this->staticChild();
    }

    /**
     * @return ChildNode<ParentNode>
     */
    public function typeChild(): ChildNode
    {
		return $this->typeChild();
    }

    public function doParentThing(): string
    {
        return 'hello';
    }
}

echo (new ParentNode())
    ->thisChild()
    ->doChildThing()
    ->end()
    ->doParentThing()
;

echo (new ParentNode())
    ->staticChild()
    ->doChildThing()
    ->end()
    ->doParentThing()
;

echo (new ParentNode())
    ->typeChild()
    ->doChildThing()
    ->end()
    ->doParentThing()
;
