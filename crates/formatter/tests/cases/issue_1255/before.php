<?php

/**
 * @immutable
 */
final readonly class Color
{
    /**
     * Creates an ANSI-256 color from a palette index.
     *
     * @throws Exception\InvalidArgumentException If $code is not in the range 0-255.
     */
    public static function ansi256(int $code): self
    {
        if ($code < 0 || $code > 255) {
            throw new Exception\InvalidArgumentException('Expected an ANSI-256 color code between 0 and 255, got '
            . $code
            . '.');
        }

        return new self(ColorKind::Ansi256, $code, 0, 0, 0);
    }
}
