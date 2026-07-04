<?php

declare(strict_types=1);

/**
 * @psalm-type Generator = Generator<int>
 * @mago-expect analysis:invalid-generator-return-type
 */
class LineDecoder
{
    /**
     * @return Generator
     */
    public function decodeLine(): \Generator
    {
        yield [];
    }
}
