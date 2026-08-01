<?php

declare(strict_types=1);

readonly class MyEntity
{
    public function __construct(private int|null $id) {}

    /** @mutation-free */
    public function getId(): int|null
    {
        return $this->id;
    }
}

function intToStr(int $n): string
{
    return '' . $n;
}

$entity = new MyEntity(1);

$id = $entity->getId() !== null ? intToStr($entity->getId()) : null;

interface UnstableEntity
{
    public function getId(): int|null;
}

function convertUnstableId(UnstableEntity $entity): string|null
{
    /** @mago-expect analysis:possibly-null-argument */
    return $entity->getId() !== null ? intToStr($entity->getId()) : null;
}

final class MutableEntity
{
    public function __construct(public int|null $id) {}

    /** @mutation-free */
    public function getId(): int|null
    {
        return $this->id;
    }

    public function clearId(): void
    {
        $this->id = null;
    }
}

function convertClearedId(MutableEntity $entity): string|null
{
    if ($entity->getId() !== null) {
        $entity->clearId();

        /** @mago-expect analysis:possibly-null-argument */
        return intToStr($entity->getId());
    }

    return null;
}

function clearIdAndReturnTrue(MutableEntity $entity): bool
{
    $entity->clearId();

    return true;
}

function convertIdClearedInCondition(MutableEntity $entity): string|null
{
    if ($entity->getId() !== null && clearIdAndReturnTrue($entity)) {
        /** @mago-expect analysis:possibly-null-argument */
        return intToStr($entity->getId());
    }

    return null;
}

function convertDirectlyClearedId(MutableEntity $entity): string|null
{
    if ($entity->getId() !== null) {
        $entity->id = null;

        /** @mago-expect analysis:possibly-null-argument */
        return intToStr($entity->getId());
    }

    return null;
}
