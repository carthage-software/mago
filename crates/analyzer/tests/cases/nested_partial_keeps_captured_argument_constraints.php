<?php

class A {}

class B {}

/** @param ($flag is true ? A : B) $value */
function select(bool $flag, A|B $value): void {}

$partial = select(flag: ?, value: new A());
$again = $partial(flag: ?);
$again(true);
// @mago-expect analysis:invalid-argument
$again(false);
