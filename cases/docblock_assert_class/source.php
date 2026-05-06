<?php

declare(strict_types=1);

class WidgetBT
{
    public int $id = 0;
}

final class GuardBT
{
    /**
     * @assert WidgetBT $value
     *
     * @throws InvalidArgumentException
     */
    public static function asWidget(mixed $value): void
    {
        if (!$value instanceof WidgetBT) {
            throw new InvalidArgumentException('not widget');
        }
    }
}

function uses_widget(mixed $v): int
{
    try {
        GuardBT::asWidget($v);

        return $v->id;
    } catch (InvalidArgumentException) {
        return 0;
    }
}

echo uses_widget(new WidgetBT());
