<?php

function g(): Generator
{
    if ((yield 1) === 2) {
        return;
    }
}

function binaryOperands($k, $v): Generator
{
    $x = (yield 1) === 2;
    $x = (yield 1) + 1;
    $x = (yield $k => $v) === 2;
    $x = (yield from h()) === 2;
    $x = (yield 1) ?? 2;
    $x = (yield 1) && true;
    $x = (yield 1) === 2;
    f((yield 1) === 2);

    return (yield 1) === 2;
}

function bareYield(): Generator
{
    $x = yield === 2;
    $x = (yield) + 1;
    $x = (yield) - 1;
}

function rightOperands($k, $v): Generator
{
    $x = 2 === yield 1;
    $x = 1 + yield 1;
    $x = 2 === yield $k => $v;
    $x = 2 === yield from h();
    $x = 2 === (yield 1) && false;
    $x = 1 + (yield 1) + 2;
    $x = !(yield 1) && false;
    $x = f(yield 1) + 2;
}

function lowPrecedence(): Generator
{
    ($x = yield 1) and false;
    ($x = yield 1) or false;
    ($x = yield 1) xor false;
}
