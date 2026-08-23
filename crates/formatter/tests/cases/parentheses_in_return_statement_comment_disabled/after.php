<?php

function complex(array $array)
{
    return (
        // The leading comment forces parentheses.
        empty($array) ? Data::fromRow(['userId' => 0, 'download' => true]) : Data::fromRow($array)
    );
}
