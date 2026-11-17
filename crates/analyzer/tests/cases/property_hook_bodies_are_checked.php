<?php

declare(strict_types=1);

final class PropertyHookBodies
{
    public int $empty {
        /** @mago-expect analysis:missing-return-statement */
        get {
        }
    }

    public int $branch {
        /** @mago-expect analysis:missing-return-statement */
        get {
            if (mt_rand(0, 1) === 1) {
                return 1;
            }
        }
    }

    public int $complete {
        get {
            if (mt_rand(0, 1) === 1) {
                return 1;
            }

            throw new RuntimeException('none');
        }
    }

    public $untyped {
        get {
        }
    }

    public string $lowered = '' {
        set => strtolower($value);
    }

    public string $wrong = '' {
        /** @mago-expect analysis:invalid-property-assignment-value */
        set => 5;
    }

    public string $widened = '' {
        set(string|int $value) => (string) $value;
    }
}
