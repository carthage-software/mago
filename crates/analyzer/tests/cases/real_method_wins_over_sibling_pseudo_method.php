<?php

declare(strict_types=1);

/**
 * @method string foo(int $x)
 */
trait PseudoTrait
{
}

trait RealTrait
{
    public function foo(?int $x = null): int
    {
        return $x ?? 1;
    }
}

final class PseudoFirst
{
    use PseudoTrait;
    use RealTrait;
}

final class RealFirst
{
    use RealTrait;
    use PseudoTrait;
}

function trait_order(PseudoFirst $a, RealFirst $b): int
{
    return $a->foo() + $b->foo();
}

/**
 * @method string bar()
 */
interface PseudoInterface
{
}

interface RealInterface
{
    public function bar(): int;
}

interface PseudoParentFirst extends PseudoInterface, RealInterface
{
}

interface RealParentFirst extends RealInterface, PseudoInterface
{
}

function interface_order(PseudoParentFirst $a, RealParentFirst $b): int
{
    return $a->bar() + $b->bar();
}

/**
 * @method string baz()
 */
class MagicParent
{
    public function __call(string $name, array $args): mixed
    {
        return 'magic';
    }
}

class NarrowingChild extends MagicParent
{
    protected function baz(): int
    {
        return 1;
    }

    public function inside(): int
    {
        return $this->baz();
    }
}
