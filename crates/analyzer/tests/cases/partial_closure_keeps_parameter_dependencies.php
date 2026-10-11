<?php
class A {} class B {}
$select = /** @param ($flag is true ? A : B) $value */
    static function (bool $flag, A|B $value): void {};
$partial = $select(flag: ?, value: new A());
$partial(true);
// @mago-expect analysis:invalid-argument
$partial(false);
