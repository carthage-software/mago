<?php

interface Element {
    /** @param Element[] $children */
    public function setChildren(array $children): self;
}

/** @phpstan-require-implements Element */
trait IsElement {
    /** @param Element[] $children */
    public function setChildren(array $children): self
    {
      return $this;
    }
}

final class CollectionElement implements Element
{
    use IsElement;
}
