<?php

namespace Relay\Event;

/**
 * Relay invalidated event class.
 */
final class Invalidated extends \Relay\Event
{
    /**
     * The type of the event represented by an integer.
     *
     * @var int
     */
    public int $type = 1;
}

/**
 * Relay flushed event class.
 */
final class Flushed extends \Relay\Event
{
    /**
     * The type of the event represented by an integer.
     *
     * @var int
     */
    public int $type = 2;
}
