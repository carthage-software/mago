<?php

declare(strict_types=1);

/**
 * A service class for managing users.
 */
class UserService
{
    /**
     * @param boolean     $active
     * @param string|null $name   The user name
     *
     * @return integer
     *
     * @throws \RuntimeException
     */
    public function findUsers(?string $name, bool $active): int
    {
        return 0;
    }

    /**
     * @type string
     */
    private string $foo;

    /**
     * @param integer     $id
     * @param string|null $email    The email address
     * @param boolean     $verified
     *
     * @return void
     */
    public function updateUser(int $id, ?string $email, bool $verified): void {}

    /** */

    /**
     * @param double $amount
     * @param real   $tax
     *
     * @return boolean
     */
    public function processPayment(float $amount, float $tax): bool
    {
        return true;
    }

    /**
     * Get user by ID.
     *
     * @param integer $id The user ID
     *
     * @return UserInterface|null The user or null
     *
     * @throws \InvalidArgumentException If invalid
     */
    public function getUser(int $id): ?UserInterface
    {
        return null;
    }
}
