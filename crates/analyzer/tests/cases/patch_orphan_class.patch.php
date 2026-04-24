<?php

// @mago-expect analysis:patch-introduces-new-symbol
class Orphan
{
    public function actuallyWantInt(int $x): void {}
}
