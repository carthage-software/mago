<?php

declare(strict_types=1);

namespace Issue2463;

/** @template T of object */
interface Repository
{
    /** @return T */
    public function find(): object;
}

/**
 * @template TEntity of object
 * @template TRepository of Repository<TEntity>
 */
abstract class Base
{
    /** @param TRepository $repository */
    public function __construct(public readonly Repository $repository) {}
}

/**
 * @template TEntity of object
 * @template TRepository of Repository<TEntity>
 * @extends Base<TEntity, TRepository>
 */
abstract class Child extends Base
{
}

/**
 * @template TObject of object
 * @template TRepo of Repository<TObject>
 * @extends Child<TObject, TRepo>
 */
abstract class RenamedChild extends Child
{
}

/**
 * @template TEntity of object
 * @template TRepository of Repository<TEntity>
 */
interface Contract
{
    /** @return TRepository */
    public function repository(): Repository;
}

/**
 * @template TEntity of object
 * @template TRepository of Repository<TEntity>
 * @extends Contract<TEntity, TRepository>
 */
interface ChildContract extends Contract
{
}

/**
 * @template TEntity of object
 * @template TRepository of Repository<TEntity>
 * @implements Contract<TEntity, TRepository>
 */
abstract class ContractImplementation implements Contract
{
}

/**
 * @template TEntity of object
 * @template TRepository of Repository<TEntity>
 */
trait RepositoryTrait
{
    /** @param TRepository $repository */
    public function __construct(public readonly Repository $repository) {}
}

/**
 * @template TEntity of object
 * @template TRepository of Repository<TEntity>
 */
abstract class TraitConsumer
{
    /** @use RepositoryTrait<TEntity, TRepository> */
    use RepositoryTrait;
}

final class Entity {}

final class OtherEntity {}

/** @implements Repository<Entity> */
final class EntityRepository implements Repository
{
    public function find(): Entity
    {
        return new Entity();
    }
}

/** @extends Base<Entity, EntityRepository> */
final class ConcreteChild extends Base
{
}

/**
 * @template TEntity of object
 * @template TRepository of Repository<object>
 * @extends Base<TEntity, TRepository>
 * @mago-expect analysis:invalid-template-parameter
 */
abstract class WiderRepositoryChild extends Base
{
}

/**
 * @template TRepository of Repository<OtherEntity>
 * @extends Base<Entity, TRepository>
 * @mago-expect analysis:invalid-template-parameter
 */
abstract class WrongEntityChild extends Base
{
}
