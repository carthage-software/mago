<?php

declare(strict_types=1);

interface IdentifiableEntity
{
    public function getId(): int;
}

interface NameableEntity
{
    public function getName(): string;
}

class CompleteEntity implements IdentifiableEntity, NameableEntity
{
    public function __construct(
        private int $id,
        private string $name,
    ) {}

    #[Override]
    public function getId(): int
    {
        return $this->id;
    }

    #[Override]
    public function getName(): string
    {
        return $this->name;
    }
}

function processIdentifiable(IdentifiableEntity $item): void
{
    echo 'ID: ' . $item->getId() . "\n";

    if ($item instanceof NameableEntity) {
        echo 'Name: ' . $item->getName() . "\n";
    }
}

function processNameable(NameableEntity $item): void
{
    echo 'Name: ' . $item->getName() . "\n";

    if ($item instanceof IdentifiableEntity) {
        echo 'ID: ' . $item->getId() . "\n";
    }
}

function describeObject(object $obj): string
{
    $parts = [];

    if ($obj instanceof IdentifiableEntity) {
        $parts[] = 'id=' . $obj->getId();
    }

    if ($obj instanceof NameableEntity) {
        $parts[] = 'name=' . $obj->getName();
    }

    if ([] === $parts) {
        return 'Unknown object';
    }

    return implode(', ', $parts);
}

function test(): void
{
    $entity = new CompleteEntity(1, 'Test Entity');

    processIdentifiable($entity);
    processNameable($entity);

    echo describeObject($entity) . "\n";
}
