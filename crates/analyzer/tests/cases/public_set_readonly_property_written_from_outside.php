<?php

declare(strict_types=1);

final class Entity // @mago-expect analysis:missing-constructor
{
    public public(set) readonly ?string $name;

    public readonly ?string $hidden;
}

final readonly class Record // @mago-expect analysis:missing-constructor
{
    public public(set) string $name;

    public string $hidden;
}

final class Hydrator
{
    public function hydrate(Entity $entity, Record $record): void
    {
        $entity->name = 'x'; // @mago-expect analysis:possibly-invalid-property-write
        $record->name = 'x'; // @mago-expect analysis:possibly-invalid-property-write
        $entity->hidden = 'x'; // @mago-expect analysis:invalid-property-write
        $record->hidden = 'x'; // @mago-expect analysis:invalid-property-write
    }
}
