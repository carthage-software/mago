<?php

declare(strict_types=1);

class A
{
    public function doA(): string
    {
        return 'hello';
    }

    public string $propA = 'value';

    public static function staticA(): int
    {
        return 42;
    }
}

/**
 * @mixin A
 */
class B
{
    public function doB(): void {}

    public function __call(string $name, array $arguments): mixed
    {
        return (new A())->$name(...$arguments);
    }
}

$b = new B();
$b->doB();
echo $b->doA();

/**
 * @template T of object
 * @mixin T
 */
class Delegatee
{
    /**
     * @param T $delegate
     */
    public function __construct(
        private object $delegate,
    ) {}

    public function __call(string $name, array $arguments): mixed
    {
        return $this->delegate->$name(...$arguments);
    }
}

$d = new Delegatee(new Exception('My message'));
echo $d->getMessage(); // Works - T resolves to Exception

/**
 * @mixin A
 */
class WithGet
{
    public function __get(string $name): mixed
    {
        return (new A())->$name;
    }

    public function __call(string $name, array $arguments): mixed
    {
        return (new A())->$name(...$arguments);
    }
}

$wg = new WithGet();
echo $wg->propA; // Should work - WithGet has @mixin A and __get
echo $wg->doA(); // Should work - WithGet has @mixin A and __call

class C
{
    public function doC(): int
    {
        return 42;
    }
}

/**
 * @mixin A
 * @mixin C
 */
class MultiMixin
{
    public function __call(string $name, array $_arguments): mixed
    {
        if ($name === 'doA') {
            return (new A())->$name();
        } elseif ($name === 'doC') {
            return (new C())->$name();
        }

        return null;
    }
}

$mm = new MultiMixin();
$mm->doA(); // Works - from A
$mm->doC(); // Works - from C

/**
 * @mixin A
 */
class StaticMixin
{
    public static function __callStatic(string $name, array $arguments): mixed
    {
        return A::$name(...$arguments);
    }
}

StaticMixin::staticA();

/**
 * @mixin A
 */
class NoCall
{
    // Missing __call - but class is not final, so subclass might have it
}

$nc = new NoCall();
$nc->doA();

/**
 * @mixin A
 */
class NoGet
{
    public function __call(string $name, array $args): mixed
    {
        return (new A())->$name(...$args);
    }

    // Missing __get
}

$ng = new NoGet();
echo $ng->propA;

/**
 * @mixin A
 */
class NoCallStatic
{
    // Missing __callStatic
}

NoCallStatic::staticA();

/**
 * @mixin A
 */
final class FinalNoCall
{
    // Missing __call - class is final, so no subclass can add it
}

$fnc = new FinalNoCall();
$fnc->doA();

/**
 * @mixin A
 */
final class FinalNoGet
{
    public function __call(string $name, array $args): mixed
    {
        return (new A())->$name(...$args);
    }
}

$fng = new FinalNoGet();
echo $fng->propA;

/**
 * @mixin A
 */
final class FinalNoCallStatic
{
    // Missing __callStatic
}

FinalNoCallStatic::staticA();

/**
 * @mixin A
 */
class LimitedMixin
{
    public function __call(string $name, array $_args): mixed
    {
        if ($name === 'doA') {
            return (new A())->$name();
        }

        return null;
    }
}

$lm = new LimitedMixin();
$lm->nonExistentMethod();
