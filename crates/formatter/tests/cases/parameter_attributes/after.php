<?php

class Foo
{
    public function __construct(
        #[BarAttr(['type' => Bar::class])]
        private Bar $bar,
        #[BazAttr(['type' => Baz::class])]
        private Baz $baz,
        #[QuxAttr(['type' => Qux::class])]
        private Qux $qux,
        #[QuuxAttr(['type' => Quux::class])]
        private Quux $quux,
    ) {}
}
