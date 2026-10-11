<?php

trait TraitFinalSameSourceActions
{
    public function run(): void {}

    public function other(): void {}
}

class TraitFinalSameSourceBase
{
    use TraitFinalSameSourceActions {
        run as final locked;
    }
}

// @mago-expect analysis:override-final-method
class TraitFinalSameSourceChild extends TraitFinalSameSourceBase
{
    use TraitFinalSameSourceActions {
        other as locked;
    }
}

class TraitFinalSameSourceOpenBase
{
    use TraitFinalSameSourceActions {
        run as open;
    }
}

class TraitFinalSameSourceOpenChild extends TraitFinalSameSourceOpenBase
{
    use TraitFinalSameSourceActions {
        other as open;
    }
}
