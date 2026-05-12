<?php

declare(strict_types=1);

namespace Test;

use function imagecreatetruecolor;
use function imagesetthickness;

class Cache
{
    public ?string $prop = null;

    public function get(): string
    {
        $_im = imagecreatetruecolor(width: 1, height: 1);
        if (null === $this->prop) {
            $this->prop = 'empty';
        }

        imagesetthickness($_im, thickness: 1);

        return $this->prop;
    }
}
