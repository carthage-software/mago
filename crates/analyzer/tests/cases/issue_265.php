<?php declare(strict_types=1);

function use_string(string $s): void
{
    echo $s;
}

function use_int(int $i): void
{
    echo $i;
}

/**
 * @template T
 */
interface testInterface
{
    /** @param array<int,T> $x */
    public function test(array $x): void;

    /** @return T|int|float */
    public function getValue(): mixed;
}

/**
 * @implements testInterface<string>
 */
class testClass implements testInterface
{
    /** @inheritDoc */
    public function test(array $x): void
    {
        foreach ($x as $int => $string) {
            use_string($string);
            use_int($int);
        }
    }

    /** @inheritDoc */
    public function getValue(): mixed
    {
        return 'test';
    }
}

$obj = new testClass();
$obj->test([1 => 'one', 2 => 'two', 3 => 'three']);
use_string((string) $obj->getValue());
