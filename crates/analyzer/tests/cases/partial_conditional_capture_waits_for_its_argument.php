<?php

class A {}

class B {}

/** @param ($flag is true ? A : B) $value */
function select(bool $flag, A|B $value): void {}

function exercise(bool $flag): void
{
    $partial = select(flag: ?, value: new A());
    $partial($flag);
    $partial(true);
    // @mago-expect analysis:invalid-argument
    $partial(false);
}
