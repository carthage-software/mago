<?php declare(strict_types=1);

class C {
  static function foo(): int { return 0; }
  function bar(): int { return 0; }
}

function useInt(int $i): void {
    echo $i;
}

$arr_static = [C::class , 'foo'];
$str_static = 'C::foo';

useInt($arr_static());
useInt($str_static());
