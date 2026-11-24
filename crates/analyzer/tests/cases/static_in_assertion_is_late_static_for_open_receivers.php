<?php

declare(strict_types=1);

class LateStaticUser
{
    /** @psalm-assert-if-true static $x */
    public static function is(object $x): bool
    {
        return $x instanceof static;
    }

    /** @psalm-assert-if-true static $x */
    public function same(object $x): bool
    {
        return $x instanceof static;
    }

    public function name(): string
    {
        return 'reached';
    }
}

final class LateStaticAdmin extends LateStaticUser {}

final class LateStaticOther {}

/** @param class-string<LateStaticUser> $class */
function late_static_on_class_string(string $class, LateStaticUser|LateStaticOther $o): string
{
    if ($class::is($o)) {
        return 'is';
    }

    if ($o instanceof LateStaticUser) {
        return $o->name();
    }

    return 'other';
}

function late_static_on_object(LateStaticUser $u, LateStaticUser|LateStaticOther $o): string
{
    if ($u->same($o)) {
        return 'same';
    }

    if ($o instanceof LateStaticUser) {
        return $o->name();
    }

    return 'other';
}

/** @psalm-consistent-constructor */
class LateStaticBox
{
    /**
     * @psalm-assert static $x
     *
     * @throws Exception
     */
    public function __construct(object $x)
    {
        if (!$x instanceof static) {
            throw new Exception();
        }
    }

    /** @throws Exception */
    public static function make(object $o): static
    {
        new static($o);

        return $o;
    }
}

interface LateStaticComparable
{
    /** @psalm-assert-if-true static $object */
    public function equals(LateStaticComparable $object): bool;
}

final class LateStaticMoney implements LateStaticComparable
{
    /** @psalm-assert-if-true static $object */
    public function equals(LateStaticComparable $object): bool
    {
        return $object instanceof self;
    }
}

function late_static_interface(LateStaticComparable $left, LateStaticComparable $right): bool
{
    return $left->equals($right);
}

function late_static_final(LateStaticMoney $left, LateStaticMoney $right): bool
{
    return $left->equals($right);
}
