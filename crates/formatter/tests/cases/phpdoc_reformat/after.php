<?php

declare(strict_types=1);

/**
 * A service class for managing users.
 */
class UserService
{
    /**
     * @param bool        $active
     * @param string|null $name   The user name
     *
     * @return int
     *
     * @throws \RuntimeException
     */
    public function findUsers(?string $name, bool $active): int
    {
        return 0;
    }

    /**
     * @var string
     */
    private string $foo;

    /**
     * @param int         $id
     * @param string|null $email    The email address
     * @param bool        $verified
     *
     * @return void
     */
    public function updateUser(int $id, ?string $email, bool $verified): void {}

    /** */

    /**
     * @param float $amount
     * @param float $tax
     *
     * @return bool
     */
    public function processPayment(float $amount, float $tax): bool
    {
        return true;
    }

    /**
     * Get user by ID.
     *
     * @param int $id The user ID
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
