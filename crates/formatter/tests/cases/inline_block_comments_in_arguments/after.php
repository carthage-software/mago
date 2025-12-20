<?php

// Block comments trailing arguments should stay with their arguments
foo(4 /* length */, 8 /* height */);

// Multiple inline block comments
bar($a /* comment a */, $b /* comment b */, $c /* comment c */);

// Mixed: inline and trailing semicolon
baz($x /* inline */); /* trailing */

// Comment only followed by comma, not semicolon
qux(1 /* one */, 2 /* two */, 3 /* three */);

class Example
{
    private function addFooooos($foooooIds, /*string*/ $precision)
    {
    }
}
