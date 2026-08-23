<?php

function complex(array $array)
{
    return empty($array) ? Data::fromRow(['userId' => 0, 'download' => true, 'force' => false, 'another' => 15.6]) : Data::fromRow($array);
}
