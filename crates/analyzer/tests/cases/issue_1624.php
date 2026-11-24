<?php

class SelfRef
{
    /** @mago-expect analysis:unresolvable-class-constant */
    const int b = self::b;
}

class StaticRef
{
    /** @mago-expect analysis:unresolvable-class-constant */
    const int b = static::b;
}

// Multi-step cycle: must not crash, and is reported.
class CycleA
{
    /** @mago-expect analysis:self-referencing-constant */
    const int x = CycleB::y;
}

class CycleB
{
    /** @mago-expect analysis:self-referencing-constant */
    const int y = CycleA::x;
}

// Sanity: a non-cyclic forward reference must still resolve normally.
class Ok
{
    const int FIRST = 1;
    const int SECOND = self::FIRST;
}
