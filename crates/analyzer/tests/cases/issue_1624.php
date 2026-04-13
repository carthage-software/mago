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

// Multi-step cycle: must not crash, even though we don't (yet) report it.
class CycleA
{
    const int x = CycleB::y;
}

class CycleB
{
    const int y = CycleA::x;
}

// Sanity: a non-cyclic forward reference must still resolve normally.
class Ok
{
    const int FIRST = 1;
    const int SECOND = self::FIRST;
}
