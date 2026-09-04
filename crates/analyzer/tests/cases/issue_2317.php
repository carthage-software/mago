<?php

declare(strict_types=1);

namespace Issue2317;

interface Element {}

interface Concrete extends Element {}

/** @template T */
interface Box
{
    /** @return T */
    public function first(): mixed;
}

/** @template T of Element */
interface Container
{
    /** @return Box<T> */
    public function elements(): Box;

    /** @param Box<T> $elements */
    public function setElements(Box $elements): void;
}

/** @require-implements Container */
trait ContainerTrait {}

/** @require-implements Container<Concrete> */
trait ConcreteContainerTrait {}

/** @implements Container<Concrete> */
final class ConcreteContainer implements Container
{
    use ContainerTrait;

    /** @var Box<Concrete> */
    private Box $box;

    /** @param Box<Concrete> $box */
    public function __construct(Box $box)
    {
        $this->box = $box;
    }

    /** @return Box<Concrete> */
    public function elements(): Box
    {
        return $this->box;
    }

    /** @param Box<Concrete> $elements */
    public function setElements(Box $elements): void
    {
        $this->box = $elements;
    }
}

/** @implements Container<Concrete> */
final class ExplicitConcreteContainer implements Container
{
    use ConcreteContainerTrait;

    /** @var Box<Concrete> */
    private Box $box;

    /** @param Box<Concrete> $box */
    public function __construct(Box $box)
    {
        $this->box = $box;
    }

    /** @return Box<Concrete> */
    public function elements(): Box
    {
        return $this->box;
    }

    /** @param Box<Concrete> $elements */
    public function setElements(Box $elements): void
    {
        $this->box = $elements;
    }
}
