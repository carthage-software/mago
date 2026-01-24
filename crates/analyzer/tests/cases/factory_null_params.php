<?php

declare(strict_types=1);

class Entity
{
    // @mago-expect analysis:write-only-property
    private ?int $id = null;
    // @mago-expect analysis:write-only-property
    private string $name;
    // @mago-expect analysis:write-only-property
    private ?DateTimeImmutable $dateCreated = null;
    // @mago-expect analysis:write-only-property
    private ?DateTimeImmutable $dateUpdated = null;

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
        ?int $id = 1,
        ?string $name = null,
        ?DateTimeImmutable $dateCreated = null,
        ?DateTimeImmutable $dateUpdated = null,
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

    $entity2 = EntityFactory::make(id: 42, name: 'Custom Name', dateCreated: new DateTimeImmutable('2024-01-01'));

    $entity3 = EntityFactory::make(dateUpdated: new DateTimeImmutable('2024-12-01'));
}
