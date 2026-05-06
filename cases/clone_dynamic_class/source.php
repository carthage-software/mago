<?php

declare(strict_types=1);

/**
 * @consistent-constructor
 */
abstract class BaseEntity
{
    final public function __construct(
        protected int $id,
    ) {}

    public function getId(): int
    {
        return $this->id;
    }
}

final class Item extends BaseEntity
{
    public function getName(): string
    {
        return "Item {$this->id}";
    }
}

final class Product extends BaseEntity
{
    public function getPrice(): float
    {
        return $this->id * 10.0;
    }
}

class Cloner
{
    /**
     * @param BaseEntity $original
     */
    public function cloneEntity(BaseEntity $original): BaseEntity
    {
        $className = $original::class;

        return new $className($original->getId());
    }
}

function test(): void
{
    $cloner = new Cloner();

    $item = new Item(1);
    $clonedItem = $cloner->cloneEntity($item);

    echo $clonedItem->getId();
}
