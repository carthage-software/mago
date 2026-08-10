<?php

declare(strict_types=1);

class Foo
{
    /** @var array<string, string> */
    private array $messages = [];

    public function setValue(mixed $value): void
    {
        $this->messages = [];

        if ($value === 'a') {
            $this->messages['foo'] = 'A';
        }

        if ($value === 'b') {
            $this->messages['foo'] = 'B';
        }
    }

    /** @return array<string, string> */
    public function get(): array
    {
        return $this->messages;
    }
}

$foo = new Foo();

$foo->setValue('fred');
doAssert($foo->get()['foo'] ?? null, null);

$foo->setValue('a');
doAssert($foo->get()['foo'] ?? null, 'A');

$foo->setValue('b');
doAssert($foo->get()['foo'] ?? null, 'B');

/**
 * @template Expect
 * @param Expect $expect
 * @assert =Expect $value
 * @throws Exception
 */
function doAssert(mixed $value, mixed $expect): void
{
    if ($value === $expect) {
        return;
    }

    throw new Exception('Nope');
}
