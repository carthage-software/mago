<?php

declare(strict_types=1);

namespace App;

abstract class Entity
{
    public function __construct(public readonly int $id)
    {
    }
}

/**
 * Library base class (generic).
 *
 * @template TEntity of Entity
 */
abstract class AbstractMapper
{
    /** @var array<int, TEntity> */
    protected array $identityMap = [];

    /**
     * @param TEntity $entity
     */
    protected function storeInIdentityMap(Entity $entity): void
    {
        $this->identityMap[$entity->id] = $entity;
    }

    /**
     * @return TEntity|null
     */
    public function fromIdentityMap(int $id): ?Entity
    {
        return $this->identityMap[$id] ?? null;
    }
}

/**
 * Project layer: forwards its own template to the library class.
 *
 * @template TEntity of Entity
 * @extends AbstractMapper<TEntity>
 */
abstract class ProjectMapper extends AbstractMapper
{
}

final class Product extends Entity
{
}

// Two-level inheritance without @extends tag:
// TEntity must resolve to bound Entity, so $this->storeInIdentityMap($product) is valid.
/**
 * @mago-expect analysis:missing-template-parameter
 */
final class ProductMapper extends ProjectMapper
{
    public function create(int $id): Product
    {
        $product = new Product($id);
        $this->storeInIdentityMap($product);

        return $product;
    }
}

// Single-level inheritance without @extends tag:
/**
 * @mago-expect analysis:missing-template-parameter
 */
final class OtherMapper extends AbstractMapper
{
    public function create(int $id): Product
    {
        $product = new Product($id);
        $this->storeInIdentityMap($product);

        return $product;
    }
}

/**
 * Three-level inheritance:
 *
 * @template TEntity of Entity
 * @extends ProjectMapper<TEntity>
 */
abstract class DeepProjectMapper extends ProjectMapper
{
}

/**
 * @mago-expect analysis:missing-template-parameter
 */
final class DeepProductMapper extends DeepProjectMapper
{
    public function create(int $id): Product
    {
        $product = new Product($id);
        $this->storeInIdentityMap($product);

        return $product;
    }
}

/**
 * Narrowed bound in intermediate layer:
 *
 * @template TEntity of Product
 * @extends AbstractMapper<TEntity>
 */
abstract class NarrowedProjectMapper extends AbstractMapper
{
}

/**
 * @mago-expect analysis:missing-template-parameter
 */
final class NarrowedProductMapper extends NarrowedProjectMapper
{
    public function create(int $id): Product
    {
        $product = new Product($id);
        $this->storeInIdentityMap($product);

        return $product;
    }

    public function invalidEntity(Entity $entity): void
    {
        // @mago-expect analysis:less-specific-argument
        $this->storeInIdentityMap($entity);
    }
}
