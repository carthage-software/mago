 <?php

 /**
  * @param callable(...):void $callable
  */
 function queue(callable $callable): void
 {
     $callable();
 }

 queue(function (): void {});
 queue(function (string $x): string {
     return $x;
 });
 queue(function (string $x, string $y): string {
     return $x . $y;
 });
