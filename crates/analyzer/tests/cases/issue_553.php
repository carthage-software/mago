<?php declare(strict_types=1);

/**
 * @psalm-consistent-constructor
 * @method static foo()
 */
class MagoTest
{
    public function c(self $x): self
    {
        echo 'hello';

        return $x;
    }

    public function __call(string $_name, array $_args): static
    {
        return new static();
    }
}

$a = new MagoTest();
$a->c($a->c($a->foo()));
