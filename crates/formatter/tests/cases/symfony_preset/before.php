<?php

declare(strict_types=1);

namespace App\Service;

use App\Entity\User;
use App\Repository\UserRepository;
use Symfony\Component\HttpFoundation\Request;

class UserService
{
    private UserRepository $repository;

    private string $defaultLocale;

    public function __construct(UserRepository $repository, string $defaultLocale = 'en')
    {
        $this->repository = $repository;
        $this->defaultLocale = $defaultLocale;
    }

    // Empty method bodies should NOT be inlined
    public function initialize(): void
    {
    }

    protected function reset(): void
    {
    }

    // Method chain - same line style
    public function findActiveUsers(): array
    {
        return $this->repository->createQueryBuilder('u')
            ->where('u.active = :active')
            ->setParameter('active', true)
            ->orderBy('u.name', 'ASC')
            ->getQuery()
            ->getResult();
    }

    // Arrow function with space before parens
    public function getNames(array $users): array
    {
        return array_map(fn ($user) => $user->getName(), $users);
    }

    // Trailing commas in multiline parameter lists
    public function createUser(
        string $name,
        string $email,
        string $role = 'ROLE_USER',
    ): User {
        $user = new User();
        $user->setName($name);
        $user->setEmail($email);
        $user->setRole($role);

        return $user;
    }

    // Empty line before return
    public function processRequest(Request $request): array
    {
        $data = $request->getContent();
        $parsed = json_decode($data, true);

        return $parsed ?? [];
    }

    // Preserved breaking conditional
    public function determineStatus(User $user): string
    {
        return $user->isActive()
            ? 'active'
            : 'inactive';
    }

    // Short method chain stays on one line
    public function count(): int
    {
        return $this->repository->count([]);
    }

    // Concatenation without spaces in @Symfony default
    public function getFullName(string $first, string $last): string
    {
        return $first.' '.$last;
    }

    // Preserved breaking parameter list
    public function updateUser(
        User $user,
        string $name,
        string $email,
    ): void {
        $user->setName($name);
        $user->setEmail($email);
        $this->repository->save($user);
    }
}

function standaloneFunction(): void
{
}

class EmptyController
{
}
