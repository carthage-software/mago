<?php

declare(strict_types=1);

function flow_try_catch_broad_throwable(Throwable $throwable): string
{
    try {
        try {
            throw $throwable;
        } catch (Exception $exception) {
            throw $exception;
        } catch (Error) { // @mago-expect analysis:avoid-catching-error
        }

        throw new Error();
    } catch (Error) { // @mago-expect analysis:avoid-catching-error
        return 'caught';
    } catch (Throwable) {
        return 'other';
    }
}
