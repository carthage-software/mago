<?php

class A {}

class B {}

/** @param ($flag is true ? A : B) $value */
function select(bool $flag, A|B $value, int $spare): void {}

$partial = select(flag: ?, value: new A(), spare: ?);
// @mago-expect analysis:invalid-argument
$again = $partial(flag: false, spare: ?);
$again(1);
