<?php

class Something
{
    public function some(): void
    {
        if (
            ! // is array
            (
                is_array($prompts)
                // && is array of string (only)
                && empty(array_reduce(
                    $prompts,
                    fn($carry, $item) => $carry || !is_string($item),
                    false,
                ))
            )
        ) {
            return;
        }
    }
}
