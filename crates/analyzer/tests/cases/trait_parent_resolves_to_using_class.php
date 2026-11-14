<?php

declare(strict_types=1);

class TraitParent0P {}

trait TraitParent0T
{
    public function adopt(parent $other): string
    {
        return $other::class;
    }
}

final class TraitParent0C extends TraitParent0P
{
    use TraitParent0T;
}

/** @param callable(TraitParent0P): string $f */
function trait_parent_0_run(callable $f, TraitParent0P $p): string
{
    return $f($p);
}

$c0 = new TraitParent0C();
echo trait_parent_0_run($c0->adopt(...), $c0);

$f0 = $c0->adopt(...);
echo $f0(new TraitParent0P());

class TraitParent1P {}

trait TraitParent1T
{
    public ?parent $prop = null;
}

final class TraitParent1C extends TraitParent1P
{
    use TraitParent1T;
}

$c1 = new TraitParent1C();
$c1->prop = new TraitParent1P();
