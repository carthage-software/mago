<?php

declare(strict_types=1);

final class Counter {
    /**
     * @var positive-int
     * @mago-expect analysis:invalid-property-default-value
     */
    public int $value = 0;
}
