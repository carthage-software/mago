<?php

final class Issue2505DeadSwitchChainFailure extends Exception
{
}

function unreachable_switch_empty_case_chain_does_not_store_or_invoke_callable(): void
{
    switch (true) {
        case 0:
        case "":
            $callback = /** @throws Issue2505DeadSwitchChainFailure */ static function (): void {
                throw new Issue2505DeadSwitchChainFailure();
            };
            $callback();
        default:
            break;
    }
}
