<?php

class Entity
{
    public bool $flag = false;
}

class EntityManager
{
    /**
     * @var array<int, Entity>
     */
    private array $storage = [];

    public function getEntity(int $id): Entity
    {
        $entity = $this->storage[$id] ?? new Entity();

        $this->storage[$id] = $entity;

        return $entity;
    }
}

function update_flag(EntityManager $manager, int $id): void
{
    $entity = $manager->getEntity($id);
    $entity->flag = true;
}

/**
 * @assert true $flag
 *
 * @throws \Exception
 */
function assert_true(bool $flag): void
{
    if (!$flag) {
        throw new Exception();
    }
}

/**
 * @assert false $flag
 *
 * @throws \Exception
 */
function assert_false(bool $flag): void
{
    if ($flag) {
        throw new Exception();
    }
}

/**
 * @throws \Exception
 */
function some_test(EntityManager $manager): void
{
    $entity = $manager->getEntity(1);
    assert($entity->flag === false, 'expected false flag');
    update_flag($manager, 1);
    assert($entity->flag === true, 'expected true flag');
}

/**
 * @throws \Exception
 */
function some_test2(EntityManager $manager): void
{
    $entity = $manager->getEntity(1);
    assert_false($entity->flag);
    update_flag($manager, 1);
    assert_true($entity->flag);
}

some_test(new EntityManager());
some_test2(new EntityManager());
