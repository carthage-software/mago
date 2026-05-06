 <?php

 /**
  * @param array<string, string> $array
  * @return Generator<string>
  */
 function generator(array $array): Generator
 {
     yield $array['key'] ?? 'default';
 }
