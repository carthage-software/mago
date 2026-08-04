<?php

declare(strict_types=1);

/**
 * @template TEntity of object
 */
class EntityRepository
{
    /**
     * @var TEntity|null
     */
    public ?object $entity = null;
}

final class UserEntity {}

/**
 * @extends EntityRepository<UserEntity>
 */
final class UserRepository extends EntityRepository {}

/**
 * @template TEntity of object
 *
 * @param class-string<EntityRepository<TEntity>>|null $repositoryClass
 *
 * @return TEntity
 *
 * @throws RuntimeException
 */
function configureEntity(?string $repositoryClass): object
{
    throw new RuntimeException($repositoryClass ?? '');
}

function takeUserEntity(UserEntity $entity): void
{
    unset($entity);
}

configureEntity(null);

// `UserRepository` specializes `TEntity` through its `@extends`, so the argument is
// valid and the return type is `UserEntity`.
takeUserEntity(configureEntity(UserRepository::class));

/**
 * @template TKey of array-key
 * @template TValue
 */
final class Collection
{
    /**
     * @param array<TKey, TValue> $items
     * @param class-string<Collection<TKey, TValue>> $collectionClass
     */
    public function __construct(
        public array $items,
        public string $collectionClass = Collection::class,
    ) {}

    /**
     * @return TValue
     *
     * @throws RuntimeException
     */
    public function first(): mixed
    {
        foreach ($this->items as $item) {
            return $item;
        }

        throw new RuntimeException('empty');
    }
}

function takeInt(int $value): void
{
    unset($value);
}

// Naming the very class being parameterized carries no specialization, so it must not
// widen the parameters that `$items` already pinned.
$collection = new Collection(['a' => 1]);

takeInt($collection->first());
