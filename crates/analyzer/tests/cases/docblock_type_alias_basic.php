<?php

declare(strict_types=1);

/**
 * @phpstan-type UserId int
 * @phpstan-type UserName non-empty-string
 */
final class UserA
{
    /**
     * @param UserId $id
     * @param UserName $name
     */
    public function __construct(
        public int $id,
        public string $name,
    ) {}

    /**
     * @return UserId
     */
    public function getId(): int
    {
        return $this->id;
    }

    /**
     * @return UserName
     */
    public function getName(): string
    {
        return $this->name;
    }
}

$user = new UserA(1, 'alice');
echo $user->getId();
echo $user->getName();
