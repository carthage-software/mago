<?php

$a = $this->viewCache->getCachedViewPath(
    path: $view->path,
    compiledView: fn() => $this->cleanupCompiled($this->compiler->compile($view->path)),
);

$p = new Point(
    x: $actual + self::MARGIN_X + 1 + self::PADDING_X + 2,
    y: self::MARGIN_TOP + $this->offsetY,
);

$t = new TailReader()->tail(
    path: $debugLogPath,
    format: fn(string $text) => $this->highlighter->parse($text, new VarExportLanguage()),
);

$this->console->keyValue(
    key: $cacheClass,
    value: "<style='bold fg-green'>CLEARED</style>",
);
