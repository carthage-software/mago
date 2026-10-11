<?php
class Closure {
    public static function getCurrent(): Closure { exit(0); }
}
class Base {}
class Child extends Base {
    public function copy(Base $value): Base {
        $copy = static function (parent $item, int $depth): parent {
            if ($depth <= 0) { return $item; }
            $recurse = Closure::getCurrent();
            return $recurse($item, $depth - 1);
        };
        return $copy($value, 1);
    }
}
