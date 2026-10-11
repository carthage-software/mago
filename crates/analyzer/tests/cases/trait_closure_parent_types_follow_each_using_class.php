<?php
class FirstBase {}
class SecondBase {}
trait Copier {
    /** @return Closure(parent): parent */
    public function copier(): Closure {
        return static fn (parent $item): parent => $item;
    }
}
class First extends FirstBase { use Copier; }
class Second extends SecondBase { use Copier; }
function first(FirstBase $value): FirstBase {
    $copy = (new First())->copier();
    return $copy($value);
}
function second(SecondBase $value): SecondBase {
    $copy = (new Second())->copier();
    return $copy($value);
}
