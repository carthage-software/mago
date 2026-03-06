<?php

declare(strict_types=1);

/**
 *
 * A service class for managing users.
 *
 *
 */
class UserService
{
    /**
     * @throws \RuntimeException
     * @param boolean $active
     * @return integer
     * @param null|string $name The user name
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
     * @param integer $id
     * @param null|string $email   The email address
     * @param boolean $verified
     * @return void
     */
    public function updateUser(int $id, ?string $email, bool $verified): void {}

    /** */

    /**
     * @param double $amount
     * @param real $tax
     * @return boolean
     */
    public function processPayment(float $amount, float $tax): bool
    {
        return true;
    }

    /**
     * Get user by ID.
     *
     * @param integer $id  The user ID
     * @throws \InvalidArgumentException If invalid
     * @return null|UserInterface  The user or null
     */
    public function getUser(int $id): ?UserInterface
    {
        return null;
    }
}
