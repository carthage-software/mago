<?php

declare(strict_types=1);

final class Person
{
    public function __construct(
        public string $email,
        public string $name,
    ) {}
}

/**
 * @param array<string, Person> $_accounts
 */
function register_accounts(array $_accounts): void {}

/**
 * @param array<string, Person> $people
 *
 * @return array<string, Person>
 */
function seed_accounts(array $people): array
{
    $accounts = [];

    foreach ($people as $person) {
        $accounts[$person->email] = $person;
    }

    $last = end($accounts);
    if ($last === false) {
        return $accounts;
    }

    prev($accounts);
    next($accounts);
    reset($accounts);

    register_accounts($accounts);

    return $accounts;
}
