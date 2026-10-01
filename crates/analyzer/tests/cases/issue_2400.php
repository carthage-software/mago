<?php

declare(strict_types=1);

namespace Issue2400;

class Adder
{
    #[\NoDiscard]
    public function __invoke(): int
    {
        return 1;
    }

    #[\NoDiscard]
    public function add(): int
    {
        return 1;
    }
}

final class InheritedAdder extends Adder {}

final class MustUseAdder
{
    /** @must-use */
    public function __invoke(): int
    {
        return 1;
    }
}

final class OrdinaryAdder
{
    public function __invoke(): int
    {
        return 1;
    }
}

$o = new Adder();
// @mago-expect analysis:unused-method-call
$o();
// @mago-expect analysis:unused-method-call
$o->__invoke();
// @mago-expect analysis:unused-method-call
$o->add();

$m = new MustUseAdder();
// @mago-expect analysis:unused-method-call
$m();
// @mago-expect analysis:unused-method-call
$m->__invoke();

$inherited = new InheritedAdder();
// @mago-expect analysis:unused-method-call
$inherited();

// @mago-expect analysis:unused-method-call
($o());
// @mago-expect analysis:unused-method-call
(new MustUseAdder())();

$firstClass = $o->add(...);
// @mago-expect analysis:unused-method-call
$firstClass();

$callableArray = [$o, 'add'];
// @mago-expect analysis:unused-method-call
$callableArray();

function discardUnion(OrdinaryAdder|MustUseAdder $adder): void
{
    // @mago-expect analysis:unused-method-call
    $adder();
}

/**
 * @template T of Adder
 * @param T $adder
 */
function discardGeneric(Adder $adder): void
{
    // @mago-expect analysis:unused-method-call
    $adder();
}

function discardUnknown(callable $callback): void
{
    $callback();
}

discardUnion($m);
discardGeneric($o);
discardUnknown($m);

echo $o();
echo $m();
echo $inherited();

(void) $o();
(void) $m();
(void) $inherited();

$ordinary = new OrdinaryAdder();
$ordinary();

$closure = static fn (): int => 1;
$closure();

$noDiscardClosure = #[\NoDiscard] static fn (): int => 1;
// @mago-expect analysis:unused-function-call
$noDiscardClosure();
echo $noDiscardClosure();
(void) $noDiscardClosure();

$mustUseClosure = /** @must-use */ static function (): int {
    return 1;
};
// @mago-expect analysis:unused-function-call
$mustUseClosure();
