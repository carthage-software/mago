<?php

class Sequence
{
    private const PERFECT_NUMBERS = [
        6,
        28,
        496,
        8128,
        33550336,
        8589869056,
        137438691328,
        2305843008139952128,
        2658455991569831744654692615953842176,
        191561942608236107294793378084303638130997321548169216,
    ];

    /**
     * @return list<int|float>
     */
    public function getPerfectNumbers(): array
    {
        return self::PERFECT_NUMBERS;
    }
}
