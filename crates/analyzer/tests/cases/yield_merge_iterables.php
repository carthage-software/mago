 <?php

 /**
  * @template KLeft
  * @template KRight
  * @template VLeft
  * @template VRight
  *
  * @param iterable<KLeft, VLeft> $lhs
  * @param iterable<KRight, VRight> $rhs
  *
  * @return Generator<KLeft|KRight, VLeft|VRight>
  */
 function merge_iterable(iterable $lhs, iterable $rhs): Generator
 {
     foreach ($lhs as $key => $value) {
         yield $key => $value;
     }

     foreach ($rhs as $key => $value) {
         yield $key => $value;
     }
 }

 /**
  * @return iterable<string, string>
  */
 function get_string_string_iterable(): iterable
 {
     return [
         'key1' => 'value1',
         'key2' => 'value2',
     ];
 }

 /**
  * @return iterable<int, string>
  */
 function get_int_string_iterable(): iterable
 {
     return [
         1 => 'value1',
         2 => 'value2',
     ];
 }

 function i_take_string(string $_string): void
 {
 }

 function i_take_int_or_string(string|int $_value): void
 {
 }

 $merged = merge_iterable(get_string_string_iterable(), get_int_string_iterable());

 foreach ($merged as $key => $value) {
     i_take_int_or_string($key);
     i_take_string($value);
 }
