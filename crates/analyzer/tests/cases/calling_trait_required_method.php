 <?php

 interface SomeInterface
 {
     public function example(): string;
 }

 /**
  * @require-implements SomeInterface
  */
 trait SomeInterfaceTrait
 {
     public function call(): void
     {
         echo $this->example();
     }
 }

 class SomeClass
 {
     public function example(): string
     {
         return 'Hello, World!';
     }
 }

 /**
  * @require-extends SomeClass
  */
 trait SomeClassTrait
 {
     public function call(): void
     {
         echo $this->example();
     }
 }
