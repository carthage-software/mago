<?php

function unmatched_switch_case_does_not_keep_catch_reachable(callable $dynamicCallable): void
{
    try {
        switch (true) {
            case false:
                $dynamicCallable();
                break;
            default:
                break;
        }
    } catch (LogicException) {
    }
}

function unknown_switch_subject_keeps_case_reachable(callable $dynamicCallable, bool $condition): void
{
    try {
        switch ($condition) {
            case false:
                $dynamicCallable();
                break;
            default:
                break;
        }
    } catch (LogicException) {
    }
}
