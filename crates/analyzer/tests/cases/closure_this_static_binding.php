<?php

declare(strict_types=1);

class Widget {}

/** @param-closure-this Widget $callback */
function bind_widget(callable $callback): void {}

// @mago-expect analysis:undefined-variable
bind_widget(static fn () => $this);
// @mago-expect analysis:undefined-variable
bind_widget(static function () {
    return $this;
});
