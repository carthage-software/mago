<?php

/**
 * @param object{number: int, str: string} $obj
 * @mago-expect analysis:type-confirmation
 */
function basic_object_test(object $obj): void
{
    Mago\confirm($obj, 'object{number: int, str: string}');
    $num = $obj->number;
    $str = $obj->str;

    Mago\confirm($num, 'int');
    Mago\confirm($str, 'string');
}
