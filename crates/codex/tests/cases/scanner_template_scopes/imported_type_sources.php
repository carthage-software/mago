<?php

namespace App;

use Lib\Source;

/**
 * @phpstan-import-type Payload from Source as LocalPayload
 * @template Source
 * @method LocalPayload magic()
 */
class Container
{
    /** @return LocalPayload */
    public function value() {}
}
