 <?php

 class A
 {
     public const int Foo = 1;
     public const string Bar = 'bar';
 }

 enum B
 {
     /**
      * @var list<int>
      */
     public const array Foo = [1, 2, 3];

     case Bar;
 }

 /**
  * @param A|B|class-string<A>|enum-string<B> $c
  * @param 'Foo'|'Bar' $const
  *
  * @return int|string|list<int>|B
  */
 function get_constant(string|object $c, string $const): int|string|array|B
 {
     $value = $c::{$const};

     return $value;
 }

 $_int = get_constant(A::class, 'Foo'); // int(1)
 $_string = get_constant(A::class, 'Bar'); // string(3) "bar"
 $_array = get_constant(B::class, 'Foo'); // array(3) { [0]=> int(1) [1]=> int(2) [2]=> int(3) }
 $_enum = get_constant(B::class, 'Bar'); // enum(B::Bar)
