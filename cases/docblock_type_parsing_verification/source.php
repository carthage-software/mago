 <?php

 function get_bool(): bool
 {
     return true;
 }

 /**
  * @return array{
  *   algorithm: 'bcrypt',
  *   options: array{'cost': int<4, 31>}
  * }
  */
 function get_bcrypt_info(): array
 {
     return [
         'algorithm' => 'bcrypt',
         'options' => [
             'cost' => 10,
         ],
     ];
 }

 /**
  * @return array{
  *   algorithm: 'argon2i',
  *   options: array{memory_cost: int<1, max>, 'time_cost': int<1, max>, threads?: int<1, max>}
  * }
  */
 function get_argon2_info(): array
 {
     return [
         'algorithm' => 'argon2i',
         'options' => [
             'memory_cost' => 65536,
             'time_cost' => 4,
             'threads' => 1,
         ],
     ];
 }

 /**
  * @return array{
  *   algorithm: 'bcrypt'|'argon2i',
  *   options: array{cost?: int<4, 31>, 'memory_cost'?: int<1, max>, 'time_cost'?: int<1, max>, 'threads'?: int<1, max>}
  * }
  */
 function get_one_of_the_two(): array
 {
     if (get_bool()) {
         return get_bcrypt_info();
     }

     return get_argon2_info();
 }
