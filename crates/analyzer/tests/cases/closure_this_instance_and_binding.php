<?php

declare(strict_types=1);

class Widget
{
    public int $value = 1;

    public function run(): void
    {
        $arrow = fn (): int => $this->value;
        $closure = function (): int {
            return $this->value;
        };

        if ($this instanceof Child) {
            $this->onlyOnChild();
        }
    }

    public static function withBinding(): void
    {
        bind_widget(fn (): int => $this->value);
        bind_widget(function (): int {
            return $this->value;
        });
    }
}

class Child extends Widget
{
    public function onlyOnChild(): void {}
}

/** @param-closure-this Widget $callback */
function bind_widget(callable $callback): void {}
