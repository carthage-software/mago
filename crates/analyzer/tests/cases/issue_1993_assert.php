<?php

declare(strict_types=1);

/** @psalm-mutation-free */
readonly class User
{
    public function __construct(
        public ?DateTimeImmutable $d1 = null,
        public ?DateTimeImmutable $d2 = null,
    ) {}
}

final class Test
{
    public function foo(User $user): void
    {
        assert($user->d1 !== null);
        assert($user->d2 !== null);

        $this->printDate($user->d1);
        $this->printDate($user->d2);
    }

    private function printDate(DateTimeImmutable $d): void
    {
        print_r($d);
    }
}
