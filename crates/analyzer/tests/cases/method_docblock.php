<?php

declare(strict_types=1);

/**
 * @method void a($a) no type in param
 * @method int e($a) no type in param
 * @method int b(int $a) with types
 * @method static int c(int $a) with static
 * @method static int d(callable(int): int $callable) with static and callable param
 * @method private static int f() private method
 */
class Test {}

$t = new Test();

$t->a(10);
$e = $t->e(10);
/** @mago-expect analysis:type-confirmation */
Mago\confirm($e, 'int');
$b = $t->b(10);
/** @mago-expect analysis:type-confirmation */
Mago\confirm($b, 'int');
$c = Test::c(10);
/** @mago-expect analysis:type-confirmation */
Mago\confirm($c, 'int');
$z = Test::d(fn(int $p) => $p * 2);
/** @mago-expect analysis:type-confirmation */
Mago\confirm($z, 'int');

/** @mago-expect analysis:too-few-arguments */
Test::c();

/** @mago-expect analysis:too-many-arguments */
Test::c(1, 2, 3);

/** @mago-expect analysis:non-existent-method */
Test::x();

/** @mago-expect analysis:invalid-static-method-access */
Test::a(10);

/** @mago-expect analysis:invalid-method-access */
Test::f();

class X
{
    public function __call($name, $arguments) {}

    public static function __callStatic($name, $arguments) {}
}

$x = new X();

/** @mago-expect analysis:non-documented-method */
$x::x();

/** @mago-expect analysis:non-documented-method */
$x->x();
