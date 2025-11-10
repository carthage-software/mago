<?php

/**
 * @phpstan-type UserId = int
 * @phpstan-type UserName = string
 */
class User
{
    /**
     * @param UserId $id
     * @param UserName $name
     */
    public function __construct(int $id, string $name)
    {
        echo $id;
        echo $name;
    }

    /**
     * @return UserName
     */
    public function getName()
    {
        return 'example';
    }

    /**
     * @return UserId
     */
    public function getId()
    {
        return 1;
    }
}

/**
 * @phpstan-import-type UserId from User
 * @phpstan-import-type UserName from User as Name
 */
class UserService
{
    /**
     * @param UserId $id
     * @param Name $name
     */
    public function save(int $id, string $name): void
    {
        echo $id;
        echo $name;
    }
}

$user = new User(123, 'Alice');
$user2 = new User($user->getId(), $user->getName());

$service = new UserService();
$service->save(456, 'Bob');
$service->save($user->getId(), $user->getName());
$service->save($user2->getId(), $user2->getName());
