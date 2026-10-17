<?php

function later_explicit_case_prevents_default_from_reaching_catch(callable $dynamicCallable): void
{
    try {
        switch (2) {
            default:
                $dynamicCallable();
                break;
            case 3:
                $dynamicCallable();
                break;
            case 2:
                break;
        }
    } catch (LogicException) {
    }
}
