<?php

declare(strict_types=1);

function get_val(): mixed
{
    return 1;
}

function i_take_int(int $a): void
{
    echo $a;
}

/**
 * @method void a($a) no type in param
 * @method int e($a) no type in param
 * @method int b(int $a) with types
 * @method static int c(int $a) with static
 * @method static int d(callable(int): int $callable) with static and callable param
 * @method private static int _f() private method
 */
class Test
{
    /** @param array<mixed> $arguments */
    public function __call(string $name, array $arguments): mixed
    {
        return get_val();
    }

    /** @param array<mixed> $arguments */
    public static function __callStatic(string $name, array $arguments): mixed
    {
        return get_val();
    }
}

$t = new Test();

$t->a(10);
$e = $t->e(10);
i_take_int($e);
$b = $t->b(10);
i_take_int($b);
$c = Test::c(10);
i_take_int($c);
$z = Test::d(fn(int $p) => $p * 2);
i_take_int($z);

/** @mago-expect analysis:too-few-arguments */
Test::c();

/** @mago-expect analysis:too-many-arguments */
Test::c(1, 2, 3);

/** @mago-expect analysis:non-documented-method */
Test::x();

/** @mago-expect analysis:invalid-static-method-access */
Test::a(10);

/** @mago-expect analysis:invalid-method-access */
Test::_f();

class X
{
    public function __call($name, $arguments)
    {
    }

    public static function __callStatic($name, $arguments)
    {
    }
}

$x = new X();

/** @mago-expect analysis:non-documented-method */
$x::x();

/** @mago-expect analysis:non-documented-method */
$x->x();
