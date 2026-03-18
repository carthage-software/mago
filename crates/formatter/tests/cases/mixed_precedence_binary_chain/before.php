<?php

class Interval
{
    public function inSeconds(): int
    {
        return (
            ($this->years * 365 * 24 * 60 * 60)
            + ($this->months * 30 * 24 * 60 * 60)
            + ($this->weeks * 7 * 24 * 60 * 60)
            + ($this->days * 60 * 60 * 24)
            + ($this->hours * 60 * 60)
            + ($this->minutes * 60)
            + $this->seconds
        );
    }

    public function inMilliseconds(): int
    {
        return $this->years * 365 * 24 * 60 * 60 * 1000
            + $this->months * 30 * 24 * 60 * 60 * 1000
            + $this->weeks * 7 * 24 * 60 * 60 * 1000
            + $this->days * 60 * 60 * 24 * 1000
            + $this->hours * 60 * 60 * 1000
            + $this->minutes * 60 * 1000
            + $this->seconds * 1000
            + $this->milliseconds;
    }

    public function simple(): int
    {
        return $a * $b + $c * $d;
    }
}
