<?php

class A {}

/**
 * @property-read A|int $x
 * @property int $y
 * @property-write int $z
 */
class T {
 private A|int $x;
}

$t = new T();

$a = $t->x;
/** @mago-expect analysis:invalid-property-write */
$t->x = 10;

$b = $t->y;
$t->y = 10;

/** @mago-expect analysis:undefined-variable */
$c = $t->z;
$t->z = 10;
/** @mago-expect analysis:type-confirmation */
Mago\confirm($z, 'int');

