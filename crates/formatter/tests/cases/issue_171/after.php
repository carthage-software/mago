<?php

/**
 * A test case with the original reported issue, and other cases to prevent regressions.
 */
class Issue171
{
    #[TestWith(['foo_bar', '_bar', 'foo'])]
    #[TestWith(['foo.bar/', '/', 'foo.bar'])]
    // #[TestWith(['foo.bar/', ['/', 'bar/'], 'foo.bar'])]
    // #[TestWith(['foo.bar/', ['bar/', '/'], 'foo.'])]
    public function test_strip_end(string $input, string|array $strip, string $output): void
    {
        $this->assertEquals($output, str($input)->stripEnd($strip)->toString());
    }

    function y($x) {}

    // #[X]
    function y($x) {}

    // #[X]
    // #[X]
    function y($x) {}

    #[X]
    // #[X]
    function y($x) {}

    #[X]
    #[X]
    // #[X]
    // #[X]
    function y($x) {}

    #[X]
    #[X]
    #[X]
    #[X]
    function y($x) {}

    function y() {}

    // #[X]
    function y() {}

    // #[X]
    // #[X]
    function y() {}

    #[X]
    // #[X]
    function y() {}

    #[X]
    #[X]
    // #[X]
    // #[X]
    function y() {}

    #[X]
    #[X]
    #[X]
    #[X]
    function y() {}
}

function y() {}

// #[X]
function y() {}

// #[X]
// #[X]
function y() {}

#[X]
// #[X]
function y() {}

#[X]
#[X]
// #[X]
// #[X]
function y() {}

#[X]
#[X]
#[X]
#[X]
function y() {}
