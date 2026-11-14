<?php

declare(strict_types=1);

/**
 * @psalm-inheritors SealedConcreteFoo|SealedConcreteBar
 */
class SealedConcreteBase {}

final class SealedConcreteFoo extends SealedConcreteBase {}

final class SealedConcreteBar extends SealedConcreteBase {}

function sealed_concrete_f(SealedConcreteBase $v): void
{
    if (!$v instanceof SealedConcreteFoo) {
        if (!$v instanceof SealedConcreteBar) {
            echo 'plain SealedConcreteBase';
        }
    }
}

