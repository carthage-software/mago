 <?php

 /**
  * @return null|list<mixed>
  */
 function to_list(mixed $value): null|array
 {
     if (!is_array($value) || !array_is_list($value)) {
         return null;
     } else {
         return $value;
     }
 }
