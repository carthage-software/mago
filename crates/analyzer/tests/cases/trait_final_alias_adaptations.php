<?php

trait TraitFinalActions
{
    public function run(): void {}
}

class TraitFinalBase
{
    use TraitFinalActions {
        run as final locked;
    }
}

class TraitFinalOriginalOverride extends TraitFinalBase
{
    public function run(): void {}
}

class TraitFinalAliasOverride extends TraitFinalBase
{
    // @mago-expect analysis:override-final-method
    public function locked(): void {}
}

class TraitFinalMiddle extends TraitFinalBase {}

class TraitFinalGrandchild extends TraitFinalMiddle
{
    // @mago-expect analysis:override-final-method
    public function locked(): void {}
}

class TraitFinalLocalWins
{
    use TraitFinalActions {
        run as final locked;
    }

    public function locked(): void {}
}

class TraitFinalLocalChild extends TraitFinalLocalWins
{
    public function locked(): void {}
}
