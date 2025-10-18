<?php

declare(strict_types=1);

/**
 * @psalm-inheritors A|B|C|D|E|F|G|H|I|J|K|L|M|N|O
 */
abstract class Letter
{
    abstract public function getPosition(): int;
}

class A extends Letter
{
    public function getPosition(): int
    {
        return 1;
    }
}

class B extends Letter
{
    public function getPosition(): int
    {
        return 2;
    }
}

class C extends Letter
{
    public function getPosition(): int
    {
        return 3;
    }
}

class D extends Letter
{
    public function getPosition(): int
    {
        return 4;
    }
}

class E extends Letter
{
    public function getPosition(): int
    {
        return 5;
    }
}

class F extends Letter
{
    public function getPosition(): int
    {
        return 6;
    }
}

class G extends Letter
{
    public function getPosition(): int
    {
        return 7;
    }
}

class H extends Letter
{
    public function getPosition(): int
    {
        return 8;
    }
}

class I extends Letter
{
    public function getPosition(): int
    {
        return 9;
    }
}

class J extends Letter
{
    public function getPosition(): int
    {
        return 10;
    }
}

class K extends Letter
{
    public function getPosition(): int
    {
        return 11;
    }
}

class L extends Letter
{
    public function getPosition(): int
    {
        return 12;
    }
}

class M extends Letter
{
    public function getPosition(): int
    {
        return 13;
    }
}

class N extends Letter
{
    public function getPosition(): int
    {
        return 14;
    }
}

class O extends Letter
{
    public const POSITION = 15;

    public function getPosition(): int
    {
        return 15;
    }
}

function describeLetter(Letter $letter): string
{
    if ($letter instanceof A) {
        return 'Letter A at position ' . $letter->getPosition();
    }

    if ($letter instanceof B) {
        return 'Letter B at position ' . $letter->getPosition();
    }

    if ($letter instanceof C) {
        return 'Letter C at position ' . $letter->getPosition();
    }

    if ($letter instanceof D) {
        return 'Letter D at position ' . $letter->getPosition();
    }

    if ($letter instanceof E) {
        return 'Letter E at position ' . $letter->getPosition();
    }

    if ($letter instanceof F) {
        return 'Letter F at position ' . $letter->getPosition();
    }

    if ($letter instanceof G) {
        return 'Letter G at position ' . $letter->getPosition();
    }

    if ($letter instanceof H) {
        return 'Letter H at position ' . $letter->getPosition();
    }

    if ($letter instanceof I) {
        return 'Letter I at position ' . $letter->getPosition();
    }

    if ($letter instanceof J) {
        return 'Letter J at position ' . $letter->getPosition();
    }

    if ($letter instanceof K) {
        return 'Letter K at position ' . $letter->getPosition();
    }

    if ($letter instanceof L) {
        return 'Letter L at position ' . $letter->getPosition();
    }

    if ($letter instanceof M) {
        return 'Letter M at position ' . $letter->getPosition();
    }

    if ($letter instanceof N) {
        return 'Letter N at position ' . $letter->getPosition();
    }

    // After 14 checks, we know it must be O
    return 'Letter O at position ' . $letter->getPosition();
}
