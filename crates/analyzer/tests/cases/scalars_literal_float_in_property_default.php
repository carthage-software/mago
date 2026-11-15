<?php

declare(strict_types=1);

final class Volume {
    public float $value = 0.0;
}

/** @mago-expect analysis:unused-statement */
new Volume();
