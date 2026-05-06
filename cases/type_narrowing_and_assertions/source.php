 <?php

 /**
  * @template Tk
  * @template Tv
  * @template T
  *
  * @param iterable<Tk, Tv> $iterable Iterable to be mapped over
  * @param (Closure(Tv): T) $function
  *
  * @return ($iterable is non-empty-array ? non-empty-list<T> : list<T>)
  */
 function map(iterable $iterable, Closure $function): array
 {
     if (is_array($iterable)) {
         return array_values(array_map($function, $iterable));
     }

     $result = [];
     foreach ($iterable as $value) {
         $result[] = $function($value);
     }

     return $result;
 }
