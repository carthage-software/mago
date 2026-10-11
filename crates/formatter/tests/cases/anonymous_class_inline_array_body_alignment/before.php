<?php

final class Example
{
    public static function cases(): array
    {
        return [
            'keyed empty class' => [new class {
            }, []],
            [self::getGuardEvent(new class {
            }), true],
            [self::getEvent(new class {
            }), 0], // Empty context
            [self::getEvent(new class {
            }, Workflow::DEFAULT_INITIAL_CONTEXT), 0], // Explicit context
            'populated class' => [new class {
                public function value(): int
                {
                    return 1;
                }
            }, []],
        ];
    }
}
