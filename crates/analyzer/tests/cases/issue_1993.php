<?php

declare(strict_types=1);

class Assertion
{
    /**
     * @pure
     *
     * @psalm-assert !null $value
     *
     * @throws \InvalidArgumentException
     */
    public static function notNull(mixed $value, string $message = ''): void
    {
        if ($value === null) {
            throw new \InvalidArgumentException($message);
        }
    }
}

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
    /**
     * @throws \InvalidArgumentException
     */
    public function foo(User $user): void
    {
        Assertion::notNull($user->d1);
        Assertion::notNull($user->d2);

        $this->printDate($user->d1);
        $this->printDate($user->d2);
    }

    private function printDate(DateTimeImmutable $d): void
    {
        print_r($d);
    }
}
