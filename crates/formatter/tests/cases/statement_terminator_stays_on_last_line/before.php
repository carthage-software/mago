<?php

use Vendor\Package\Component\Subcomponent\Service\LongNamespaceName\SomeVeryLongClassNameHere as Alias;
use Vendor\Package\A, Vendor\Package\B, Vendor\Package\C, Vendor\Package\D, Vendor\Package\LongerName;

echo 'Could not store the data for the article with the number: '
    . $articleNumber
    . ', the reason given by the database was: '
    . implode(', ', $statement->errorInfo())
    . "\n";

echo some_function([
    'key1' => 'value1',
    'key2' => 'value2',
    'key3' => 'value3',
    'key4' => 'value4',
    'key5' => 'value5',
    'key6' => 'value6',
]);
