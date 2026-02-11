<?php

declare(strict_types=1);

abstract class AbstractEntity {}

/**
 * @template T of AbstractEntity
 */
interface RepositoryInterface
{
    /**
     * @return T|null
     */
    public function get(int $id): ?AbstractEntity;

    /**
     * @return list<T>
     */
    public function getAll(): array;

    /**
     * @param T $entity
     */
    public function delete(AbstractEntity $entity): void;

    /**
     * @param T $entity
     */
    public function save(AbstractEntity $entity): void;
}

/**
 * @template T of AbstractEntity
 *
 * @implements RepositoryInterface<T>
 */
abstract class AbstractRepository implements RepositoryInterface
{
    /**
     * @return T|null
     */
    #[Override]
    public function get(int $id): ?AbstractEntity
    {
        return $this->get($id);
    }

    /**
     * @return list<T>
     */
    #[Override]
    public function getAll(): array
    {
        return $this->getAll();
    }

    /**
     * @param T $entity
     */
    #[Override]
    public function delete(AbstractEntity $entity): void
    {
        $this->delete($entity);
    }

    /**
     * @param T $entity
     */
    #[Override]
    public function save(AbstractEntity $entity): void
    {
        $this->save($entity);
    }
}

class User extends AbstractEntity {}

/**
 * @extends RepositoryInterface<User>
 */
interface UserRepositoryInterface extends RepositoryInterface
{
    public function findByEmail(string $email): ?User;

    public function findByGoogleId(string $googleId): ?User;
}

/**
 * @extends AbstractRepository<User>
 */
final class UserRepository extends AbstractRepository implements UserRepositoryInterface
{
    #[Override]
    public function findByEmail(string $email): ?User
    {
        return $this->findByEmail($email);
    }

    #[Override]
    public function findByGoogleId(string $googleId): ?User
    {
        return $this->findByGoogleId($googleId);
    }
}
