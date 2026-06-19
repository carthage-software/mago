<?php

$a = new class(?, 2) {};

$b = new class($first, ?, named: ?, ...) {
    public int $x = 1;
};

$c = new class(?) extends Base implements Contract {
    public function foo(): void {}
};
