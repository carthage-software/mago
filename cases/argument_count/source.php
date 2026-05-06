 <?php

 final readonly class Number
 {
     public function __construct(
         private int $value,
     ) {}

     public function getValue(): int
     {
         return $this->value;
     }
 }

 final readonly class Calculator
 {
     public static function sum(Number $first, Number ...$rest): Number
     {
         $total = $first->getValue();
         foreach ($rest as $number) {
             $total += $number->getValue();
         }

         return new Number($total);
     }
 }

 function sum_numbers(Number ...$number): Number
 {
     return $number === [] ? new Number(0) : Calculator::sum(...$number);
 }

 $a = new Number(10);
 $b = new Number(20);
 $c = new Number(30);

 $_result = Calculator::sum(...[$a, $b, $c]);
 $_result = Calculator::sum($a, ...[$b, $c]);
 $_result = Calculator::sum($a, $b, ...[$c]);
 $_result = Calculator::sum($a, $b, $c);
