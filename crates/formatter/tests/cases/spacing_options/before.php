<?php

// Declare
declare(strict_types = 1);

// Use statements
use Foo\Bar;
use Foo\Baz;
use function Foo\bar;
use function Foo\baz;
use const Foo\A;
use const Foo\B;

// Control structures
if ($condition) {
    // ...
}

for ($i = 0; $i < 10; $i++) {
    // ...
}

foreach ($items as $item) {
    // ...
}

while ($condition) {
    // ...
}

try {
    // ...
} catch (Exception $e) {
    // ...
}

switch ($value) {
    // ...
}

match ($value) {
    // ...
};

// Closures and functions
$closure = function ($x) use ($y) {
    return $x + $y;
};

function foo($param1, $param2) {
    return $param1 . $param2;
}

class MyClass {
    public function bar($param1, $param2) {
        return $param1 * $param2;
    }
}

// Class-like structures
class Foo {
    public static $bar;
    public function baz() {
        return 'baz';
    }
}

interface Bar {
    public function qux();
}

trait Baz {
    public function quux() {
        return 'quux';
    }
}

enum Qux: string {
    case A = 'a';
    case B = 'b';
}

// Method chains
$result = $obj
    ->method1()
    ->method2();

// Binary operators
$a = $b + $c;
$d = $e - $f;
$g = $h * $i;
$j = $k / $l;
$m = $n % $o;
$p = $q ** $r;
$s = $t << $u;
$v = $w >> $x;
$y = $z . $aa;
$bb = $cc ?? $dd;
$ee = $ff ?: $gg;
$hh = $ii && $jj;
$kk = $ll || $mm;
$nn = $oo == $pp;
$qq = $rr === $ss;
$tt = $uu != $vv;
$ww = $xx !== $yy;
$zz = $aaa < $bbb;
$ccc = $ddd > $eee;
$fff = $ggg <= $hhh;
$iii = $jjj >= $kkk;
$lll = $mmm <=> $nnn;
$ooo = $ppp & $qqq;
$rrr = $sss | $ttt;
$uuu = $vvv ^ $www;

// Unary operators
$a = + $b;
$c = - $d;
$e = ! $f;
$g = ~ $h;
$i = ++ $j;
$k = -- $l;
$l = (int) $m;
$n = & $o;
$p = @ $q;
$r = ++ $s;
$t = -- $u;

// Arrays
$array = [
    ['foo',  1.2,  123, false],
    ['bar',  52.4, 456, true],
    ['baz',  3.6,  789, false],
    ['qux',  4.8,    1, true],
    ['quux', 5.0,   12, false],
];

// Null type hints
function nullableFunction(?string $param): ?int {
    return null;
}

function unionTypeFunction(string | int $param): string | int {
    return $param;
}

function intersectionTypeFunction(Foo & Bar $param): Foo & Bar{
    return $param;
}

// New expressions
$obj = new Foo();
$result = (new Foo)->baz();

// Exit and die
exit();
die();

// Attributes
#[SomeAttribute()]
class MyClassWithAttribute {}

// Named arguments
function namedArgs(string $name ='foo', int $age = 30) {}

namedArgs(name: 'bar', age: 25);

// List destructuring
list ($a, $b) = [1, 2];

// Legacy array
$legacyArray = array ($a, $b);

// Array access
$value = $array[ $key ];

// Grouping parenthesis
$result = ( $a + $b ) * $c;
