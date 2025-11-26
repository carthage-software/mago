<?php

declare(strict_types=1);

/**
 * @type Fixtures array{
 *  'customer.1': int,
 *  'customer.2': int,
 *  'customer.3': int,
 *  'account.1': string
 * }
 */
class Foo
{
    /** @var Fixtures */
    protected static array $fixture;

    public function foo(): void
    {
        $params = [
            'customer.1',
            'customer.2',
            'customer.3',
        ];

        foreach ($params as $id) {
            $this->takesInt(self::$fixture[$id]);
        }
    }

    private function takesInt(int $id): void
    {
    }
}
