<?php

declare(strict_types=1);

class Entity
{
    private null|int $id = null;
    private string $name;
    private null|DateTimeImmutable $dateCreated = null;
    private null|DateTimeImmutable $dateUpdated = null;

    public function __construct(string $name)
    {
        $this->name = $name;
    }

    public function setId(int $id): void
    {
        $this->id = $id;
    }

    public function setDateCreated(DateTimeImmutable $date): void
    {
        $this->dateCreated = $date;
    }

    public function setDateUpdated(DateTimeImmutable $date): void
    {
        $this->dateUpdated = $date;
    }
}

class EntityFactory
{
    public static function make(
        null|int $id = 1,
        null|string $name = null,
        null|DateTimeImmutable $dateCreated = null,
        null|DateTimeImmutable $dateUpdated = null,
    ): Entity {
        $entity = new Entity($name ?? 'Default Name');

        if ($id !== null) {
            $entity->setId($id);
        }

        if ($dateCreated !== null) {
            $entity->setDateCreated($dateCreated);
        }

        if ($dateUpdated !== null) {
            $entity->setDateUpdated($dateUpdated);
        }

        return $entity;
    }
}

function test(): void
{
    $entity1 = EntityFactory::make();

    $entity2 = EntityFactory::make(
        id: 42,
        name: 'Custom Name',
        dateCreated: new DateTimeImmutable('2024-01-01'),
    );

    $entity3 = EntityFactory::make(dateUpdated: new DateTimeImmutable('2024-12-01'));
}
