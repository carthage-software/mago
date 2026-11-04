<?php

// Control structures
if ($condition) {
    // ...
}

for ($i = 0; $i < 10; $i++) {
    // ...
}

foreach ($items as $item) {
    // ...
}

while ($condition) {
    // ...
}

try {
    // ...
} catch (Exception $e) {
    // ...
}

switch ($value) {
    // ...
}

match ($value) {
    // ...
};

// Closures and functions
$closure = function ($x) use ($y) {
    return $x + $y;
};

function foo($param1, $param2) {
    return $param1 . $param2;
}

class MyClass {
    public function __construct(
        private readonly string $property,
    ) {}
}

// Arrays
$array = [
    ['foo',  1.2,  123, false],
    ['bar',  52.4, 456, true],
    ['baz',  3.6,  789, false],
    ['qux',  4.8,    1, true],
    ['quux', 5.0,   12, false],
];

// Named arguments
function namedArgs(string $name ='foo', int $age = 30) {}

namedArgs(name: 'bar', age: 25);
namedArgs(
    name: 'bar',
    age: 25,
);

// List destructuring
list ($a, $b) = [1, 2];
[$a, $b] = [1, 2];

// Legacy array
$legacyArray = array ($a, $b);

// Array access
$value = $array[$key];
$value = $array[0];
$value = $array['property'];

// Language functions
unset($value);
if(isset($value)) print $value;
die();
die('End of script');

// Grouping parenthesis
$result = ($a + $b) * $c;