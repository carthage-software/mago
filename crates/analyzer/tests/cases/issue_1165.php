<?php

declare(strict_types=1);

class Param {}

// Setup: interface has non-nullable @param, base class widens to nullable.
interface ParamInterface {
    /** @param Param $x */
    public function process(Param $x): void;
}

abstract class Base implements ParamInterface {
    /** @param Param|null $x */
    public function process(?Param $x = null): void {}
}

// gap=0: direct child (regression guard)
class DirectChild extends Base {
    /** @inheritDoc */
    public function process(?Param $x = null): void {}
}

// gap=1: one non-overriding intermediate (core bug from #1165)
class Middle extends Base {}
class GapChild extends Middle {
    /** @inheritDoc */
    public function process(?Param $x = null): void {}
}

// gap=2: two non-overriding intermediates
class DeepMiddle extends Middle {}
class DeepGapChild extends DeepMiddle {
    /** @inheritDoc */
    public function process(?Param $x = null): void {}
}

// True positive: real contravariance violation through gap.
abstract class NullableBase {
    public function execute(?Param $x): void {}
}
class NullableMiddle extends NullableBase {}
class NarrowingChild extends NullableMiddle {
    // @mago-expect analysis:incompatible-parameter-type
    public function execute(Param $x): void {}
}
