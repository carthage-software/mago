<?php

declare(strict_types=1);

class User
{
    public function __construct(
        public ?DateTime $d1 = null,
        public ?DateTime $d2 = null,
    ) {}
}

class Test
{
    public function foo(User $user): void
    {
        assert($user->d1 !== null);
        assert($user->d2 !== null);

        $this->printDate($user->d1);
        $this->printDate($user->d2);
    }

    private function printDate(DateTime $d): void
    {
        print_r($d);
    }
}
