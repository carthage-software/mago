<?php

// Test: standalone comments inside ignore regions should be preserved as-is

class StandaloneCommentInIgnoreRegion
{
    // @mago-format-ignore-start
    /**
     badly formatted
     docblock  without    asterisks
    */
    // @mago-format-ignore-end

    public function bar(): void
    {
    }
}

class MultipleStandaloneComments
{
    // @mago-format-ignore-start
    /* block comment
    with  weird   spacing */
    // a single line comment
    /**
     another badly
       formatted docblock
    */
    // @mago-format-ignore-end

    public const VALUE = 1;
}
