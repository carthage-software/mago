<?php
class Base {}
class Child extends Base {
    public function copy(Base $value): Base {
        $native = static function (parent $item): parent { return $item; };
        $documented = /**
         * @param parent $item
         * @return parent
         */
        static fn ($item) => $item;
        $partial = $native(item: ?);
        return $partial($documented($native(item: $value)));
    }
}
