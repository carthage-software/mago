<?php

declare(strict_types=1);

class User
{
    public function getId(): int
    {
        return 1;
    }
}

class Admin extends User
{
}

function supportsClass(string $class): bool
{
    return User::class === $class || is_subclass_of($class, User::class);
}

function test(): void
{
    $supports1 = supportsClass(User::class);
    $supports2 = supportsClass(Admin::class);
    $supports3 = supportsClass('SomeOther');

    echo $supports1 ? 'yes' : 'no';
    echo $supports2 ? 'yes' : 'no';
    echo $supports3 ? 'yes' : 'no';
}
