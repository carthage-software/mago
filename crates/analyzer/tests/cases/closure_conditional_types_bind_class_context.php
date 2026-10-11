<?php
class Base {}
class Child extends Base {
    public function copy(Base $base): Base {
        $select = /**
         * @param ($flag is true ? parent : self) $value
         * @return ($flag is true ? parent : self)
         */
        static function (bool $flag, Base $value): Base { exit(1); };
        $partial = $select(flag: ?, value: ?);
        $fixed = $select(flag: false, value: ?);
        require_child($select(false, $this));
        require_child($partial(false, $this));
        require_child($fixed($this));
        return $partial(true, $select(true, $base));
    }
}
function require_child(Child $value): void {}
